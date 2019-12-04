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
use std::net::Shutdown;
use network::tokens::Tokens;

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
    // child_socket => proxy_socket
    let mut proxy_map = HashMap::<Token, Token>::new();

    let mut buffer = [0 as u8; 1024];

    let mut terminate_tokens = Vec::<Token>::new();

    let mut token_generator = Tokens::new();

    // let mut buffer = ;

    // let mut copy = Vec::<u8>::new();

    loop {
        while !terminate_tokens.is_empty() {
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
                                let token = token_generator.next();
                                poll.register(&socket, token
                                              , Ready::readable()
                                              , PollOpt::edge());
                                // 先move到map中，然后进行borrow --- 抛错
                                // 可以先borrow,再move
                                let child = ChildHandler::new(&token);
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
                    // let mut handler = children_map.get_mut(&token).unwrap();
                    let socket = sockets_map.get_mut(&token).unwrap();

                    // proxy socket read
                    let is_proxy = match proxy_map.get(&token) {
                        None => false,
                        Some(server) => true,
                    };

                    let mut handler = match proxy_map.get(&token) {
                        None => children_map.get_mut(&token).unwrap(),
                        Some(child_token) => children_map.get_mut(&child_token).unwrap(),
                    };

                    if handler.before_dst_request()
                        && handler.is_dst_token_empty() {
                        let proxy_token = token_generator.next();
                        println!("proxy-token:{:?}", proxy_token.0);
                        handler.set_dst_token(proxy_token);
                    }

                    loop {
                        println!("read data:");
                        let read = socket.read(&mut buffer);
                        match read {
                            Ok(0) => {
                                println!("in read 0");
                                // sockets_map.remove(&token);
                                // children_map.remove(&token);
                                terminate_tokens.push(token);
                                socket.shutdown(Shutdown::Both);
                                println!("in 0.");
                                break;
                            }
                            Ok(size) => {
                                println!("size:{:?}", size);
                                for i in 0..size {
                                    println!("{:?},", buffer[i]);
                                    handler.receive_u8_data(buffer[i], is_proxy);
                                    buffer[i] = 0;
                                }
                            }
                            Err(e)  if e.kind() == std::io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(_) => {
                                println!("in _");
                                break;
                            }
                        }
                    }

                    match handler.handle() {
                        // TODO  add proxy socket logic
                        Ok(size) => {
                            if handler.after_dst_request() {
                                // 1. add to sockets_map
                                // 2. add to proxy_map
                                // 3. add to poll
                                let proxy_socket = handler.get_proxy_socket().unwrap();
                                let proxy_token = handler.get_dst_token().unwrap();
                                let server_token = handler.get_token();
                                sockets_map.insert(proxy_token.clone(), proxy_socket);
                                proxy_map.insert(proxy_token.clone(), server_token.clone());

                                // first register write event
                                println!("in after dst request-proxy_token:{:?}", proxy_token);
                                poll.register(sockets_map.get(proxy_token).unwrap()
                                              , *proxy_token
                                              , Ready::writable()
                                              , PollOpt::edge());
                            }

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
                    println!("in write, token:{}", token.0);
                    // let child_handler = children_map.get_mut(&token).unwrap();
                    let socket = sockets_map.get_mut(&token).unwrap();

                    //let size = child_handler.write_to_socket(socket);

                    // proxy socket read
                    let is_proxy = match proxy_map.get(&token) {
                        None => false,
                        Some(server) => true,
                    };

                    let mut child_handler = match proxy_map.get(&token) {
                        None => children_map.get_mut(&token).unwrap(),
                        Some(child_token) => children_map.get_mut(&child_token).unwrap(),
                    };

                    match child_handler.write_to_socket(socket, is_proxy) {
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
