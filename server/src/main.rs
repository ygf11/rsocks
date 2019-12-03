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

enum ReceiveType {
    Proxy,
    Server,
}

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

    let mut count = 0;

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
                    let mut handler = children_map.get_mut(&token).unwrap();
                    let socket = sockets_map.get_mut(&token).unwrap();

                    let receive_type = match proxy_map.get(&token) {
                        None => ReceiveType::Server,
                        Some(server) => ReceiveType::Proxy,
                    };

                    if handler.before_dst_request()
                        && handler.is_dst_token_empty() {
                        handler.set_dst_token(token_generator.next());
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
                                    println!("{:?}", buffer[i]);
                                    // todo server/proxy
                                    // match receive_type {
                                    //    ReceiveType::Proxy =>
                                   // }
                                    handler.receive_u8_data(buffer[i]);
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

                                poll.register(sockets_map.get(proxy_token).unwrap()
                                                , proxy_token.clone()
                                                , Ready::readable()
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
