extern crate protocol;

use mio::{Poll, Token, Ready, PollOpt};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use mio::net::{TcpStream, TcpListener};
use std::rc::Rc;
use protocol::packet::ServerStage;
use protocol::packet::*;
use self::protocol::packet::ServerStage::{Init, AuthSelectFinish, RequestFinish};
use self::protocol::packet::Version::Socks5;
use std::io::Error;


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

    pub fn init(&mut self) -> Result<Token, &'static str> {
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
    receive_buffer: Vec<u8>,
    send_buffer: Vec<u8>,
    address: Option<DstAddress>,
}

impl ChildHandler {
    pub fn new_test(forward: bool) -> ChildHandler {
        ChildHandler {
            stage: ServerStage::Init,
            forward,
            receive_buffer: Vec::<u8>::new(),
            send_buffer: Vec::<u8>::new(),
            address: None,
        }
    }
    pub fn new(forward: bool) -> ChildHandler {
        ChildHandler {
            stage: ServerStage::Init,
            forward,
            receive_buffer: Vec::<u8>::new(),
            send_buffer: Vec::<u8>::new(),
            address: None,
        }
    }

    pub fn handle(&mut self, data: &'static [u8]) -> Result<usize, &'static str> {
        let stage = &mut self.stage;
        match stage {
            ServerStage::Init => {
                let size = self.handle_init_stage(data)?;
                self.stage = AuthSelectFinish;

                println!("init stage packeg:{:?}", self.send_buffer);
                Ok(size)
            }
            ServerStage::AuthSelectFinish => {
                // parse packet and send
                let size = self.handle_dst_request(data)?;
                self.stage = RequestFinish;

                Err("err")
            }
            ServerStage::RequestFinish => {
                // receive proxy packets
                // destroy connections
                Err("err")
            }
            ServerStage::ReceiveContent => {
                // end
                Err("err")
            }

            _ => Err("unreachable.")
        }
    }

    pub fn reset(&mut self) {
        self.stage = Init;
    }

    pub fn handle_init_stage(&mut self, data: &'static [u8]) -> Result<usize, &'static str> {
        // parse packet and send
        let request = parse_auth_select_request_packet(data)?;

        check_version_type(request.version())?;

        let n_methods = request.n_methods();
        if n_methods == 0 {
            return Err("non auth method is specified.");
        }

        let methods = request.methods();
        let contains_name_pass = methods.contains(&AuthType::NamePassword);
        let contains_non = methods.contains(&AuthType::Non);

        let auth_type = if contains_name_pass {
            AuthType::NamePassword
        } else if contains_non {
            AuthType::Non
        } else {
            return Err("proxy only support non and name/password auth-method.");
        };

        let auth_select_reply = AuthSelectReply::new(Socks5, auth_type);
        let mut data = &mut encode_auth_select_reply(&auth_select_reply)?;

        self.write_to_buffer(data)
    }

    pub fn handle_dst_request(&mut self, data: &'static [u8]) -> Result<usize, &'static str> {
        let request = parse_dst_service_request(data)?;
        check_version_type(request.version())?;
        check_cmd_operation(request.cmd())?;

        let address_type = request.address_type();
        let address = request.address();
        let port = request.port();

        // save address:port
        let dst_address = transfer_address(address, address_type, port)?;
        self.address = Some(dst_address);

        Err("err")
    }


    pub fn write_to_buffer(&mut self, data: &mut Vec<u8>) -> Result<usize, &'static str> {
        let mut buffer = &mut self.send_buffer;
        let size: usize = data.len();
        buffer.append(data);

        Ok(size)
    }

    pub fn clear_send_buffer(&mut self) {
        self.send_buffer.clear()
    }

    pub fn receive_u8_data(&mut self, data: u8) -> Result<usize, &'static str> {
        let mut buffer = &mut self.receive_buffer;
        buffer.push(data);

        Ok(1)
    }

    pub fn clear_receive_buffer(&mut self) {
        self.receive_buffer.clear()
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