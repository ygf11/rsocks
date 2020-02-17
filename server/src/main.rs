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
use std::fs::read_to_string;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        panic!("address and port should be specified!");
    }

    let address = parse_address(args.get(1).unwrap());
    let port = parse_port(args.get(2).unwrap());

    //let mut address = Vec::<u8>::new();
    //address.push(127);
    //address.push(0);
    //address.push(0);
    //address.push(1);
    //let port: u16 = 10500;
    for value in &address {
        println!("{}", value);
    }

    println!("port: {}", &port);

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

    let mut buffer = [0 as u8; 1024 * 256];

    let mut terminate_tokens = Vec::<Token>::new();

    let mut token_generator = Tokens::new();

    // let mut buffer = ;

    // let mut copy = Vec::<u8>::new();

    loop {
        while !terminate_tokens.is_empty() {
            println!("terminate size:{:?}", terminate_tokens.len());
            let token = terminate_tokens.pop().unwrap();
            match proxy_map.get(&token) {
                // non-proxy
                None => {
                    let handler = match children_map.remove(&token) {
                        None => continue,
                        Some(result) => result,
                    };

                    let socket = match sockets_map.remove(&token) {
                        None => continue,
                        Some(result) => result,
                    };

                    poll.deregister(&socket);
                    socket.shutdown(Shutdown::Both);

                    let proxy_token = match handler.get_dst_token() {
                        None => continue,
                        Some(result) => result,
                    };

                    let proxy_socket = match sockets_map.remove(&proxy_token) {
                        None => continue,
                        Some(result) => result,
                    };

                    poll.deregister(&proxy_socket);
                    proxy_socket.shutdown(Shutdown::Both);
                }

                // proxy
                Some(_) => {}
            }
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
                                              , Ready::readable() | Ready::writable()
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
                    println!("read data token:{:?}", token.0);
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
                        Some(child_token) => {
                            println!("in panic point:{:?}", token.0);
                            children_map.get_mut(&child_token).unwrap()
                        }
                    };

                    //let init_proxy_env = handler.before_dst_request()
                    //    && handler.is_dst_token_empty();
                    let before_dst_request = handler.before_dst_request();
                    let empty_dst_token = handler.is_dst_token_empty();
                    let init_proxy_env = before_dst_request && empty_dst_token;

                    let mut close = false;

                    loop {
                        println!("read data:");
                        let read = socket.read(&mut buffer);
                        match read {
                            Ok(0) => {
                                //
                                close = true;
                                terminate_tokens.push(token);
                                break;
                            }
                            Ok(size) => {
                                println!("size:{:?}", size);
                                if size == 0 {
                                    continue;
                                }
                                for i in 0..size {
                                    // println!("{:?},", buffer[i]);
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

                    if close {
                        continue;
                    }
                    match handler.handle() {
                        // TODO  add proxy socket logic
                        Ok(size) => {

                            // write to client socket
                            if size != 0 {
                                println!("in write to client socket:{:?}", token);
                                match handler.forward_to_proxy() {
                                    false => {
                                        let socket = sockets_map.get_mut(&token).unwrap();
                                        handler.write_to_socket(socket, false);

                                        handler.try_enable_forward();
                                    }
                                    true => {
                                        match is_proxy {
                                            true => {
                                                handler.move_to_client();
                                                let child_token = proxy_map.get(&token).unwrap();
                                                let socket = sockets_map.get_mut(&child_token).unwrap();
                                                handler.write_to_socket(socket, false);
                                            }

                                            false => {
                                                handler.move_to_proxy();
                                                let dst_token = handler.get_dst_token().unwrap();
                                                let socket = sockets_map.get_mut(&dst_token).unwrap();
                                                handler.write_to_socket(socket, true);
                                            }
                                        }
                                    }
                                }
                            }
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

                    if init_proxy_env && !handler.proxy_inited() {
                        let proxy_socket = handler.get_proxy_socket().unwrap();
                        let proxy_token = &token_generator.next();
                        let server_token = handler.get_token();
                        sockets_map.insert(proxy_token.clone(), proxy_socket);
                        proxy_map.insert(proxy_token.clone(), server_token.clone());

                        // first register write event
                        println!("in after dst request-proxy_token:{:?}", proxy_token);
                        poll.register(sockets_map.get(proxy_token).unwrap()
                                      , *proxy_token
                                      , Ready::readable() | Ready::writable()
                                      , PollOpt::edge());

                        handler.set_proxy_inited(true);
                        handler.set_dst_token(proxy_token.clone());
                    }
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
                            println!("write all");
                            println!("local port:{:?}", socket.local_addr());
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

fn parse_address(arg: &str) -> Vec<u8> {
    let split = arg.split(".");
    let mut result = Vec::new();

    for s in split {
        let value: i32 = s.parse().unwrap();
        result.push(value as u8);
    }

    if result.len() != 4 {
        panic!("address format is not correct.");
    }

    result
}

fn parse_port(arg: &str) -> u16 {
    let value: i32 = arg.parse().unwrap();

    value as u16
}
