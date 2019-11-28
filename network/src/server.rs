use mio::{Poll, Token, Ready, PollOpt};
use std::net::{TcpListener, SocketAddr, TcpStream, IpAddr, Ipv4Addr};
use std::rc::Rc;

struct Server {
    address: Vec<u8>,
    port: u16,
    listener: Option<TcpListener>,
}

impl Server {
    fn new(address: Vec<u8>, port: u16) -> Server {
        Server {
            address,
            port,
            listener: None,
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

struct Client {
    address: Vec<u8>,
    port: u16,
    socket: Option<TcpStream>,
    count: usize,
}


impl Client {
    fn new(address: Vec<u8>, port: u16, poll: Rc<Poll>) -> Client {
        Client {
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

    fn handle_in_server(&mut self) -> Result<Token, &'static str> {
        Err("err")
    }

    fn handle_in_client(&mut self) -> Result<Token, &'static str> {
        Err("err")
    }
}
