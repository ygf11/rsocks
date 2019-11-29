extern crate protocol;

use mio::{Poll, Token, Ready, PollOpt};
use std::net::{TcpListener, SocketAddr, TcpStream, IpAddr, Ipv4Addr};
use std::rc::Rc;
use protocol::packet::ServerStage;
use protocol::packet::*;
use self::protocol::packet::ServerStage::Init;
use self::protocol::packet::Version::Socks5;

struct ServerHandler {
    address: Vec<u8>,
    port: u16,
    listener: Option<TcpListener>,
    stage: ServerStage,
}

impl ServerHandler {
    fn new(address: Vec<u8>, port: u16) -> ServerHandler {
        ServerHandler {
            address,
            port,
            listener: None,
            stage: ServerStage::Init,
        }
    }

    fn init(&mut self) -> Result<Token, &'static str> {
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

    fn accept(&mut self) -> Result<TcpStream, &'static str> {
        let (socket, remote) =
            match self.listener.as_ref().unwrap().accept() {
                Ok(stream) => Ok(stream),
                Err(err) => Err("connect failed."),
            }?;

        Ok(socket)
    }
}

struct ChildHandler {
    socket: Option<TcpStream>,
    stage: ServerStage,
    forward: bool,
    buffer: Vec<u8>,
}

impl ChildHandler {
    fn new(socket: TcpStream, forward: bool) -> ChildHandler {
        ChildHandler {
            socket:Some(socket),
            stage: ServerStage::Init,
            forward,
            buffer: Vec::<u8>::new(),
        }
    }

    fn handle(&mut self, data: &[u8]) -> Result<Token, &'static str> {
        let stage = &mut self.stage;
        match stage {
            ServerStage::Init => {}
            ServerStage::AuthSelectFinish => {
                // parse packet and send
            }
            ServerStage::RequestFinish => {}
            ServerStage::ReceiveContent => {}

            _ => unreachable!()
        }

        Err("err")
    }

    fn reset(&mut self) {
        self.stage = Init;
    }

    fn handle_init_stage(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        // parse packet and send
        let request = parse_auth_select_request_packet(data)?;
        match request.version() {
            Version::Others => return Err("version not support."),
            _ => ()
        }

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
            return Err("proxy only support non and name/password auths.");
        };

        let auth_select_reply = AuthSelectReply::new(Socks5, auth_type);
        let mut data = &mut encode_auth_select_reply(&auth_select_reply)?;

        self.write_to_buffer(data)
    }

    fn write_to_buffer(&mut self, data: &mut Vec<u8>) -> Result<usize, &'static str> {
        let mut buffer = &mut self.buffer;
        buffer.append(data);

        Ok(data.len())
    }

    fn clear_buffer(&mut self) {
        self.buffer.clear()
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