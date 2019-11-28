extern crate protocol;

use protocol::packet;
use std::collections::HashMap;
use std::time::Duration;
use std::io::Read;
use mio::{Token, Poll, Events, Ready, PollOpt};
use mio::net::{TcpStream, TcpListener};

fn main() {
    let mut count = 1;

    let mut sockets: HashMap<Token, TcpStream> = HashMap::new();
    let mut buffer = [0 as u8; 1024*1024];

    let address = "127.0.0.1:1080".parse().unwrap();
    let server = TcpListener::bind(&address).unwrap();

    let client_address = "127.0.0.1:10445".parse().unwrap();
    let client = TcpStream::connect(&client_address).unwrap();

    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);

    poll.register(&server, Token(0)
                  , Ready::readable(), PollOpt::edge());

    poll.register(&client, Token(1)
                  , Ready::readable(), PollOpt::edge());

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100))).unwrap();

        for event in events.iter() {
            match event.token() {
                Token(0) => {
                    loop {
                        println!("accept new connect.");
                        count += 1;
                        let result = server.accept();
                        match result {
                            Ok((socket, _)) => {
                                let token = Token(count);
                                poll.register(&socket, token
                                              , Ready::readable(), PollOpt::edge());
                                // 先move到map中，然后进行borrow --- 抛错
                                // 可以先borrow,再move
                                sockets.insert(token, socket);
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,

                            Err(_) => break
                        }
                    }
                }
                Token(1) => {
                    // communicate with remote server
                }

                token if event.readiness().is_readable() => {
                    // communicate with local browser
                    loop {
                        println!("read data:");
                        let mut socket = sockets.get_mut(&token).unwrap();
                        let mut read = socket.read(&mut buffer);
                        match read {
                            Ok(0) => {
                                sockets.remove(&token);
                                break;
                            }
                            Ok(size) => {
                                println!("size:{}", size);
                                println!("{:?}", buffer.get(0..10));
                                //println!("{:?}", buffer.get(1));
                                //println!("{:?}", buffer.get(2));
                            }
                            Err(e)  if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(_) => break,
                        }
                    }

                    // after read all data

                }
                _ => ()
            }
        }
    }
}
