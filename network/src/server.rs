extern crate protocol;

use mio::{Poll, Token, Ready, PollOpt};
use std::net::{TcpListener, SocketAddr, TcpStream, IpAddr, Ipv4Addr};
use std::rc::Rc;
use protocol::packet::ServerStage;

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
    socket: TcpStream,
    stage: ServerStage,
    forward: bool,
}

impl ChildHandler {
    fn new(socket: TcpStream, forward: bool) -> ChildHandler {
        ChildHandler {
            socket,
            stage: ServerStage::Init,
            forward,
        }
    }

    fn handle(&self) -> Result<Token, &'static str> {
        Err("err")
    }
}

struct ClientHandler {
    address: Vec<u8>,
    port: u16,
    socket: Option<TcpStream>,
    count: usize,

}


impl ClientHandler {
    fn new(address: Vec<u8>, port: u16, poll: Rc<Poll>) -> ClientHandler {
        ClientHandler {
            address,
            port,
            socket: None,
            count: 0,
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
}
