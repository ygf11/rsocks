use std::collections::HashMap;
use mio::Token;
use mio::net::TcpStream;
use protocol::packet::ClientStage;

pub struct Client{
    context:Context,
}

impl Client {
    fn new() -> Client {
        Client{
            context: Context::new(),
        }
    }
}

pub struct Context {
    sockets: HashMap<Token, TcpStream>,
    stage: ClientStage,
    content: Option<Vec<u8>>,
}

impl Context {
    fn new() -> Context{
       Context{
           sockets: HashMap::new(),
           stage: ClientStage::Init,
           content: None,
       }
    }
}

trait StateHandler {
    fn next_stage(&mut self);
}

impl StateHandler for Client{
    fn next_stage(&mut self){
        // todo
        let mut context =  &mut self.context;
        let mut stage = &mut context.stage;

        match stage {
            ClientStage::Init => {
                // send auth select request
                // switch AuthSelectFinish
            }
            ClientStage::SendAuthSelect => {
                // read packet
            }
            ClientStage::AuthSelectFinish => {

            }
            ClientStage::SendContentRequest => {

            }
            ClientStage::SendRequest => {

            }
            ClientStage::RequestFinish => {

            }
            ClientStage::ContentFinish => {

            }
        }


    }
}



