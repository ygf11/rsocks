extern crate protocol;
extern crate dns_lookup;

use mio::{Poll, Token, Ready, PollOpt};
use std::net::{SocketAddr, IpAddr, Ipv4Addr, SocketAddrV4};
use mio::net::{TcpStream, TcpListener};
use std::rc::Rc;
use protocol::packet::ServerStage;
use protocol::packet::*;
use self::protocol::packet::ServerStage::{Init, AuthSelectFinish, RequestFinish, ReceiveContent};
use self::protocol::packet::Version::Socks5;
use std::io::{Error, Write, ErrorKind};
use std::collections::VecDeque;
use self::protocol::packet::CmdType::Connect;
use crate::http::*;

struct DstAddress {
    ip: IpAddr,
    port: u16,
}

fn check_version_type(version: &Version) -> Result<&Version, String> {
    match version {
        Version::Socks5 => Ok(version),
        _ => Err("this version only support SOCKS5".to_string())
    }
}

fn check_cmd_operation(cmd: &CmdType) -> Result<&CmdType, String> {
    match cmd {
        CmdType::Connect => Ok(cmd),
        _ => Err("this version only support CONNECT.".to_string())
    }
}

fn connect_to_dst(address: &IpAddr, port: u16) -> Result<TcpStream, ReplyType> {
    let socket = match TcpStream::connect(
        &SocketAddr::new(*address, port)) {
        Ok(socket) => Ok(socket),
        Err(e) => {
            // todo error kind
            let kind = e.kind();
            match kind {
                _ => return Err(ReplyType::Others),
            }
        }
    }?;

    Ok(socket)
}

fn transfer_address(address: String, address_type: &AddressType)
                    -> Result<IpAddr, String> {
    match address_type {
        AddressType::Ipv4 => Ok(address.parse().unwrap()),
        AddressType::Domain => {
            let ips = match dns_lookup::lookup_host(&address) {
                Ok(list) => Ok(list),
                Err(e) => Err("err when parse domain.".to_string())
            }?;
            println!("dst ip:{:?}", ips.first());
            Ok(*ips.first().unwrap())
        }
        AddressType::Ipv6 => Err("ipv6 not support in this version.".to_string())
    }
}

pub struct ServerHandler {
    address: Vec<u8>,
    port: u16,
    listener: Option<TcpListener>,
    stage: ServerStage,
}

impl ServerHandler {
    pub fn new(address: Vec<u8>, port: u16) -> ServerHandler {
        ServerHandler {
            address,
            port,
            listener: None,
            stage: ServerStage::Init,
        }
    }

    pub fn init(&mut self) -> Result<Token, &str> {
        let vec = &self.address;
        let socket_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(vec.get(0).cloned().unwrap(),
                                     vec.get(1).cloned().unwrap(),
                                     vec.get(2).cloned().unwrap(),
                                     vec.get(3).cloned().unwrap())), self.port);

        let server = match TcpListener::bind(&socket_addr) {
            Ok(listener) => Ok(listener),
            Err(err) => Err("bind failed."),
        }?;

        let token = Token::from(0);

        self.listener = Some(server);
        Ok(token)
    }

    pub fn accept(&mut self) -> Result<(TcpStream, SocketAddr), Error> {
        self.listener.as_ref().unwrap().accept()
    }

    pub fn listener(&self) -> Option<&TcpListener> {
        self.listener.as_ref()
    }
}

pub struct ChildHandler {
    token: Token,
    stage: ServerStage,
    send_buffer: Vec<u8>,
    receive_buffer: Vec<u8>,
    dst_token: Option<Token>,
    dst_send_buffer: Vec<u8>,
    dst_receive_buffer: Vec<u8>,
    dst_socket: Option<TcpStream>,
}

impl ChildHandler {
    pub fn new_test(token: &Token) -> ChildHandler {
        ChildHandler {
            token: token.clone(),
            stage: ServerStage::Init,
            receive_buffer: Vec::<u8>::new(),
            send_buffer: Vec::<u8>::new(),
            dst_token: None,
            dst_receive_buffer: Vec::<u8>::new(),
            dst_send_buffer: Vec::<u8>::new(),
            dst_socket: None,
        }
    }
    pub fn new(token: &Token) -> ChildHandler {
        ChildHandler {
            token: token.clone(),
            stage: ServerStage::Init,
            receive_buffer: Vec::<u8>::new(),
            send_buffer: Vec::<u8>::new(),
            dst_token: None,
            dst_receive_buffer: Vec::<u8>::new(),
            dst_send_buffer: Vec::<u8>::new(),
            dst_socket: None,
        }
    }

    pub fn handle(&mut self) -> Result<usize, String> {
        let stage = &mut self.stage;
        match stage {
            ServerStage::Init => {
                let mut size;
                size = self.handle_init_stage()?;
                println!("init stage packet:{:?}", self.send_buffer);

                self.stage = ServerStage::AuthSelectFinish;
                Ok(size)
            }
            ServerStage::AuthSelectFinish => {
                // parse packet and send
                let size = self.handle_dst_request()?;
                println!("request stage packet:{:?}", self.send_buffer);

                self.stage = RequestFinish;
                Ok(size)
            }
            ServerStage::RequestFinish => {
                // receive proxy-content
                // destroy connections
                let data = self.receive_buffer.as_slice();
                let str = String::from_utf8_lossy(data);
                println!("http content size:{}", data.len());
                println!("content:{:?}", str);

                let end = match get_end_of_http_packet(
                    data, PacketType::Request, false) {
                    Ok(HttpResult::End(offset)) => offset,
                    Ok(HttpResult::DataNotEnough) => return Ok(0),
                    Err(msg) => return Err(msg)
                };

                println!("end:{}", end);
                let forward_data = Vec::<u8>::from(&data[0..end]);

                self.write_to_buffer(forward_data, true)?;

                self.stage = ReceiveContent;
                Ok(end)
            }
            ServerStage::ReceiveContent => {
                // send response data to client
                let data = self.dst_receive_buffer.as_slice();
                let str = String::from_utf8_lossy(data);
                println!("http content size:{}", data.len());
                println!("content:{:?}", str);

                let end = match get_end_of_http_packet(
                    data, PacketType::Response, false) {
                    Ok(HttpResult::End(offset)) => offset,
                    Ok(HttpResult::DataNotEnough) => return Ok(0),
                    Err(msg) => return Err(msg)
                };

                println!("reponse end:{:?}", end);

                let response = Vec::<u8>::from(&data[0..end]);
                self.write_to_buffer(response, false);


                self.stage = ServerStage::ContentFinish;
                Ok(0)
                //Err("receive content err".to_string())
            }
            ServerStage::ContentFinish => {
                // do nothing
                Ok(0)
            }

            _ => Err("unreachable err".to_string())
        }
    }

    pub fn reset(&mut self) {
        self.stage = Init;
    }

    pub fn handle_init_stage(&mut self) -> Result<usize, String> {
        let request = self.parse_auth_select_request()?;
        check_version_type(request.version())?;

        let n_methods = request.n_methods();
        if n_methods == 0 {
            return Err("non auth method is specified.".to_string());
        }

        let methods = request.methods();
        let contains_name_pass = methods.contains(&AuthType::NamePassword);
        let contains_non = methods.contains(&AuthType::Non);

        let auth_type = if contains_name_pass {
            AuthType::NamePassword
        } else if contains_non {
            AuthType::Non
        } else {
            return Err("proxy only support non and name/password auth-method.".to_string());
        };

        let auth_select_reply = AuthSelectReply::new(Socks5, auth_type);
        let data = encode_auth_select_reply(&auth_select_reply)?;
        self.clear_receive_buffer(3);

        // Ok(data.len())
        self.write_to_buffer(data, false)
    }

    pub fn parse_auth_select_request(&self) -> Result<AuthSelectRequest, String> {
        let cloned = self.receive_buffer.clone();
        let data = cloned.as_slice();
        // parse packet and send
        let request = parse_auth_select_request_packet(data)?;
        Ok(request)
    }

    pub fn handle_dst_request(&mut self) -> Result<usize, String> {
        let data = self.receive_buffer.as_slice();
        let (request, address_len) = parse_dst_service_request(data)?;

        check_version_type(request.version())?;

        // check_cmd_operation(request.cmd())?;

        let address_type = request.address_type();
        let address = request.address();
        let port = request.port();
        println!("proxy_port:{}", port);
        // save address:port
        let address_copy = String::from(&address);
        let dst_address = transfer_address(address_copy, address_type)?;

        // connect -- then return socket
        // send reply

        // 1. 检查版本 ---- 非5 直接断开
        // 2. 检查cmd ---- 如果不为connect 直接断开
        // 3. 连接远程服务 ---- 构造reply
        // 4. 将响应写入buffer
        // 5. 返回
        let reply = match request.cmd() {
            CmdType::Connect => {
                // connect
                let res = match connect_to_dst(&dst_address, port) {
                    Ok(socket) => {
                        self.dst_socket = Some(socket);
                        ReplyType::Success
                    }
                    Err(e) => e
                };

                res
            }

            _ => ReplyType::CmdNotSupport
        };

        let address_type_copy = match address_type {
            AddressType::Ipv4 => AddressType::Ipv4,
            AddressType::Ipv6 => AddressType::Ipv6,
            AddressType::Domain => AddressType::Domain,
        };

        let address_copy_2 = String::from(&address);
        let dst_reply = DstServiceReply::new(
            Version::Socks5, reply, address_type_copy, address_copy_2, port);

        let data = encode_dst_service_reply(dst_reply)?;

        self.clear_receive_buffer(address_len + 6);

        self.write_to_buffer(data, false)
    }

    pub fn clear_receive_buffer(&mut self, size: u8) {
        let mut len = size.clone();
        let buffer = &mut self.receive_buffer;
        loop {
            if len == 0 {
                break;
            }

            buffer.remove(0);
            len = len - 1;
        }
    }

    pub fn clear_send_buffer(&mut self, is_proxy: bool) {
        let mut buffer = match is_proxy {
            false => &mut self.send_buffer,
            true => &mut self.dst_send_buffer,
        };
        //let buffer = &mut self.send_buffer;
        loop {
            if buffer.is_empty() {
                break;
            }

            buffer.remove(0);
        }
    }

    pub fn write_to_buffer(&mut self, data: Vec<u8>, is_proxy: bool) -> Result<usize, String> {
        let mut buffer = match is_proxy {
            false => &mut self.send_buffer,
            true => &mut self.dst_send_buffer,
        };
        //let mut buffer = &mut self.send_buffer;

        let size: usize = data.len();

        for i in 0..data.len() {
            buffer.push(*data.get(i).unwrap());
        }

        Ok(size)
    }

    pub fn receive_u8_data(&mut self, data: u8, is_proxy: bool) -> Result<usize, &str> {
        let mut buffer = match is_proxy {
            false => &mut self.receive_buffer,
            true => &mut self.dst_receive_buffer,
        };
        //let mut buffer = &mut self.receive_buffer;
        // println!("receive_buffer len:{}", buffer.len());
        buffer.push(data);

        Ok(1)
    }

    pub fn write_to_socket(&mut self, socket: &mut TcpStream, is_proxy: bool)
                           -> Result<usize, String> {
        let buffer = match is_proxy {
            false => &self.send_buffer,
            true => &self.dst_send_buffer,
        };

        //let buffer = &self.send_buffer;

        let data = buffer.as_slice();
        let size = data.len();
        println!("send data size:{:?}", size);
        socket.write_all(data);

        self.clear_send_buffer(is_proxy);

        Ok(size)
    }

    fn buffer_dst_reply(&mut self, reply: ReplyType, address_type: AddressType
                        , address: String, port: u16) -> Result<usize, String> {
        let dst_reply = DstServiceReply::new(
            Version::Socks5, reply, address_type, address, port);

        let data = encode_dst_service_reply(dst_reply)?;

        self.write_to_buffer(data, false)
    }

    pub fn print_receive_buf_size(self) {
        println!("receive buf size:{}", self.receive_buffer.len());
    }

    pub fn before_dst_request(&self) -> bool {
        self.stage == RequestFinish
    }

    pub fn after_dst_request(&self) -> bool {
        self.stage == ReceiveContent
    }

    pub fn get_token(&self) -> &Token {
        &self.token
    }

    pub fn is_dst_token_empty(&self) -> bool {
        self.dst_token == None
    }

    pub fn set_dst_token(&mut self, token: Token) {
        self.dst_token = Some(token);
    }

    pub fn get_dst_token(&self) -> Option<&Token> {
        self.dst_token.as_ref()
    }

    pub fn get_proxy_socket(&mut self) -> Option<TcpStream> {
        self.dst_socket.take()
    }
    pub fn dst_send_buffer_empty(&self) -> bool {
        !self.dst_send_buffer.is_empty()
    }
}

struct ClientHandler {
    address: Vec<u8>,
    port: u16,
    socket: Option<TcpStream>,
    count: usize,
    stage: ClientStage,
}


impl ClientHandler {
    fn new(address: Vec<u8>, port: u16, poll: Rc<Poll>) -> ClientHandler {
        ClientHandler {
            address,
            port,
            socket: None,
            count: 0,
            stage: ClientStage::Init,
        }
    }

    fn init(&mut self) -> Result<Token, &'static str> {
        let vec = &self.address;
        let socket_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(vec.get(0).cloned().unwrap(),
                                     vec.get(1).cloned().unwrap(),
                                     vec.get(2).cloned().unwrap(),
                                     vec.get(3).cloned().unwrap())), self.port);

        let socket = TcpStream::connect(&socket_addr).unwrap();

        self.count = self.count + 1;
        self.socket = Some(socket);
        Ok(Token(self.count))
    }

    fn handle(&mut self) -> Result<Token, &'static str> {
        Err("err")
    }

    fn reset(&mut self) {
        self.stage = ClientStage::Init;
    }
}