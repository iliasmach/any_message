use actix_rt::net::{TcpStream, TcpListener};
use actix::{Actor, Addr, Context};
use crate::service::{Service, ServiceCore};
use crate::node::Node;
use crate::core::Core;
use crate::message::BaseMessage;

pub struct TcpService {
    listeners: TcpListener,
    connections: TcpStream
}

impl TcpService {

}

impl Actor for TcpService {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        todo!()
    }
}

impl Service for TcpService {
    fn config_system(&mut self, system_core: &mut ServiceCore, node: Addr<Node>) {
        todo!()
    }

    fn handle_message(&self, message: &BaseMessage) {
        todo!()
    }
}