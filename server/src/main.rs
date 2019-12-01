extern crate network;
extern crate mio;

use network::server::ChildHandler;
use network::server::ServerHandler;
use mio::{Poll, Ready, PollOpt, Events, Token};
use std::time::Duration;
use std::collections::HashMap;
use std::process::Child;
use std::io::Read;
use mio::net::TcpStream;
use mio::tcp::Shutdown;

fn main() {
    let mut address = Vec::<u8>::new();
    address.push(127);
    address.push(0);
    address.push(0);
    address.push(1);
    let port: u16 = 10500;
    let mut server = ServerHandler::new(address, port);

    let token = match server.init() {
        Ok(token) => token,
        Err(err) => {
            panic!("bind port err.")
        }
    };
    let mut poll = match Poll::new() {
        Ok(poll) => poll,
        Err(err) => {
            panic!("create poll err.")
        }
    };

    poll.register(server.listener().unwrap(), token
                  , Ready::readable() | Ready::writable(), PollOpt::edge());


    let mut events = Events::with_capacity(128);

    let mut children_map = HashMap::<Token, ChildHandler>::new();

    let mut sockets_map = HashMap::<Token, TcpStream>::new();

    let mut count = 0;

    let mut buffer = [0 as u8; 1024];

    let mut terminate_tokens = Vec::<Token>::new();
    // let mut buffer = ;

    // let mut copy = Vec::<u8>::new();

    loop {
        while !terminate_tokens.is_empty(){
            let token = terminate_tokens.pop().unwrap();
            children_map.remove(&token);
            sockets_map.remove(&token);
        }


        poll.poll(&mut events, Some(Duration::from_millis(100)));

        for event in events.iter() {
            match event.token() {
                Token(0) => {
                    loop {
                        let result = server.accept();
                        match result {
                            Ok((socket, _)) => {
                                count = count + 1;
                                let token = Token(count);
                                poll.register(&socket, token
                                              , Ready::readable()
                                              , PollOpt::edge());
                                // 先move到map中，然后进行borrow --- 抛错
                                // 可以先borrow,再move
                                let child = ChildHandler::new(true);
                                children_map.insert(token, child);
                                sockets_map.insert(token, socket);
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,

                            Err(_) => break
                        }
                    }
                }
                token if event.readiness().is_readable() => {
                    // communicate with local browser
                    let mut handler = children_map.get_mut(&token).unwrap();
                    let socket = sockets_map.get_mut(&token).unwrap();

                    loop {
                        println!("read data:");
                        let read = socket.read(&mut buffer);
                        match read {
                            Ok(0) => {
                                // sockets_map.remove(&token);
                                // children_map.remove(&token);
                                println!("in 0.");
                                break;
                            }
                            Ok(size) => {
                                println!("size:{}", size);
                                for i in 0..size {
                                    println!("{}", buffer[i]);
                                    handler.receive_u8_data(buffer[i]);
                                    buffer[i] = 0;
                                }
                            }
                            Err(e)  if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // after read all data
                                // handler.handle();
                                // handler.receive_data(&mut buffer);
                                // children_map.insert(token, handler);
                                break;
                            }
                            Err(_) => {
                                println!("in _");
                                break;
                            }
                        }
                    }

                    match handler.handle() {
                        Ok(size) => {
                            poll.reregister(sockets_map.get(&token).unwrap(), token
                                            , Ready::writable()
                                            , PollOpt::edge());
                        }
                        Err(msg) => {
                            // terminate
                            println!("read err msg:{:?}", msg);
                            terminate_tokens.push(token);
                            // children_map.remove(&token);
                            // sockets_map.remove(&token);
                            socket.shutdown(Shutdown::Both);
                        }
                    };
                }
                token if event.readiness().is_writable() => {
                    println!("in write.");
                    let child_handler = children_map.get_mut(&token).unwrap();
                    let socket = sockets_map.get_mut(&token).unwrap();

                    let size = child_handler.write_to_socket(socket);


                    match child_handler.write_to_socket(socket) {
                        Ok(size) => {
                            println!("write size:{}", size);
                            poll.reregister(sockets_map.get(&token).unwrap(), token
                                            , Ready::readable()
                                            , PollOpt::edge());
                        }
                        Err(msg) => {
                            // terminate
                            println!("write err msg:{:?}", msg);
                            terminate_tokens.push(token);
                            // children_map.remove(&token);
                            // sockets_map.remove(&token);
                            socket.shutdown(Shutdown::Both);
                        }
                    };

                }

                _ => ()
            }
        }
    }
}
