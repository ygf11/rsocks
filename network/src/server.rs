extern crate protocol;
extern crate dns_lookup;

use mio::{Poll, Token, Ready, PollOpt};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use mio::net::{TcpStream, TcpListener};
use std::rc::Rc;
use protocol::packet::ServerStage;
use protocol::packet::*;
use self::protocol::packet::ServerStage::{Init, AuthSelectFinish, RequestFinish};
use self::protocol::packet::Version::Socks5;
use std::io::{Error, Write};
use std::collections::VecDeque;


enum DstAddress {
    Ipv4(String, u16),
    Domain(String),
}

fn check_version_type(version: &Version) -> Result<&Version, &'static str> {
    match version {
        Version::Socks5 => Ok(version),
        _ => Err("this version only support SOCKS5")
    }
}

fn check_cmd_operation(cmd: &CmdType) -> Result<&CmdType, &'static str> {
    match cmd {
        CmdType::Connect => Ok(cmd),
        _ => Err("this version only support CONNECT.")
    }
}

fn transfer_address(address: String, address_type: &AddressType, port: u16)
                    -> Result<DstAddress, &'static str> {
    match address_type {
        AddressType::Ipv4 => Ok(DstAddress::Ipv4(address, port)),
        AddressType::Domain => Ok(DstAddress::Domain(address)),
        AddressType::Ipv6 => Err("ipv6 not support in this version.")
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
    stage: ServerStage,
    forward: bool,
    receive_buffer: VecDeque<u8>,
    send_buffer: Option<VecDeque<u8>>,
    address: Option<DstAddress>,
    dst_socket: Option<TcpStream>,
}

impl ChildHandler {
    pub fn new_test(forward: bool) -> ChildHandler {
        ChildHandler {
            stage: ServerStage::Init,
            forward,
            receive_buffer: VecDeque::<u8>::new(),
            send_buffer: Some(VecDeque::<u8>::new()),
            address: None,
            dst_socket: None,
        }
    }
    pub fn new(forward: bool) -> ChildHandler {
        ChildHandler {
            stage: ServerStage::Init,
            forward,
            receive_buffer: VecDeque::<u8>::new(),
            send_buffer: Some(VecDeque::<u8>::new()),
            address: None,
            dst_socket: None,
        }
    }

    pub fn handle(&mut self) -> Result<usize, String> {
        let stage = &mut self.stage;
        match stage {
            ServerStage::Init => {
                let mut size;
                size = self.handle_init_stage()?;
                println!("init stage packeg:{:?}", self.send_buffer);

                Ok(size)
            }
            ServerStage::AuthSelectFinish => {
                // parse packet and send
                let size = self.handle_dst_request()?;
                // self.stage = RequestFinish;

                Err("err".to_string())
            }
            ServerStage::RequestFinish => {
                // receive proxy packets
                // destroy connections
                Err("err".to_string())
            }
            ServerStage::ReceiveContent => {
                // end
                Err("err".to_string())
            }

            _ => Err("unreachable.".to_string())
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

        self.clear_receive_buffer();

        // Ok(data.len())
        self.write_to_buffer(data)
    }

    pub fn parse_auth_select_request(&self) -> Result<AuthSelectRequest, String> {
        let cloned = self.receive_buffer.clone();
        let (data, _) = cloned.as_slices();
        // parse packet and send
        let request = parse_auth_select_request_packet(data)?;
        Ok(request)
    }

    pub fn handle_dst_request(&mut self) -> Result<usize, String> {
        let (data, _) = self.receive_buffer.as_slices();
        let request = parse_dst_service_request(data)?;
        check_version_type(request.version())?;
        check_cmd_operation(request.cmd())?;

        let address_type = request.address_type();
        let address = request.address();
        let port = request.port();

        // save address:port
        let dst_address = transfer_address(address, address_type, port)?;
        self.address = Some(dst_address);

        // connect -- then return socket
        // send reply

        Err("err".to_string())
    }

    pub fn clear_receive_buffer(&mut self) {
        let buffer = &mut self.receive_buffer;
        loop {
            if buffer.is_empty() {
                break;
            }

            buffer.pop_front();
        }
    }

    pub fn write_to_buffer(&mut self, data: Vec<u8>) -> Result<usize, String> {
        let mut buffer = match &mut self.send_buffer {
            Some(buf) => Ok(buf),
            None => Err("send buffer is none.")
        }?;

        let size: usize = data.len();

        for i in 0..data.len() {
            buffer.push_back(*data.get(i).unwrap());
        }

        Ok(size)
    }

    pub fn receive_u8_data(&mut self, data: u8) -> Result<usize, &str> {
        let mut buffer = &mut self.receive_buffer;
        buffer.push_back(data);

        Ok(1)
    }

    pub fn write_to_socket(&mut self, socket: &mut TcpStream) -> Result<usize, String> {
        let buffer = match self.send_buffer.take() {
            Some(buf) => buf,
            None => return Ok(0),
        };

        let (data, _) = buffer.as_slices();
        socket.write_all(data);

        self.send_buffer = Some(VecDeque::new());

        Ok(data.len())
    }

    fn connect_to_dst(&mut self, address:DstAddress) -> Result<TcpStream, String>{
        match address{
            DstAddress::Ipv4(ipv4, port) => {
                let mut ipv4_addr = String::from(ipv4);
                ipv4_addr.push(':');
                ipv4_addr.push_str(&port.to_string());
                let socket = match TcpStream::connect(&ipv4_addr.parse().unwrap()){
                    Ok(client) => Ok(client),
                    Err(e) => Err("err when connect to dst server".to_string())
                }?;

                Ok(socket)
            }

            DstAddress::Domain(domain) => {
                let ips = match dns_lookup::lookup_host(&domain){
                    Ok(list) => Ok(list),
                    Err(e) => Err("err when parse domain.".to_string())
                }?;
                let ip = ips.get(0).unwrap();
                let socket = match TcpStream::connect(&SocketAddr::new(*ip, 1)){
                    Ok(client) => Ok(client),
                    Err(e) => Err("err when connect to dst server".to_string())
                }?;

                Ok(socket)
            }
        }
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