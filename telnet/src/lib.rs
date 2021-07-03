extern crate log;

use base::signal::{GetMessagesSignal, RegisterServiceInNodeSignal};
use base::error::Error;
use base::message::{Parcel, BaseMessage};
use telnet::{Telnet, TelnetEvent};
use base::route::{RouteSheet, Route};
use log::debug;
use base::service::Service;
use base::transport::Transport;
use std::time::Duration;
use base::node::Node;
use actix::prelude::*;

pub struct TelnetService {
    host: String,
    port: u16,
    buff_size: u32,
    connection: Telnet,
    buf: String,
    messages: Vec<BaseMessage>,
    ping_interval_in_millis: Option<u64>,
    self_addr: Option<Addr<Self>>,
    route: Route,
    node: Addr<Node>,
}

impl TelnetService {
    pub fn new(node: Addr<Node>, host: String, port: u16, buff_size: u32, ping_interval_in_millis: Option<u64>) -> Self {
        println!("Connection to {}:{}", host, port);

        let this = Self {
            host,
            port,
            buff_size,
            connection,
            buf: String::with_capacity(buff_size as usize),
            messages: Default::default(),
            ping_interval_in_millis,
            self_addr: None,
            route: Route::new("telnet".to_string()),
            node,
        };

        let connection = match Telnet::connect((this.host.as_str(), this.port), this.buff_size as usize) {
            Ok(conn) => conn,
            Err(_e) => {
                panic!("Error while connecting to asterisk {:?}", _e);
            }
        };

        this
    }

    pub fn read_messages(&mut self) {
        let event = match self.connection.read_timeout(Duration::from_secs(2)) {
            Ok(event) => event,
            Err(_e) => {
                debug!("Read 2 secs");
                return;
            }
        };

        match event {
            TelnetEvent::Data(buffer) => {
                let message = BaseMessage::new(buffer.to_vec(), None);
                self.messages.push(message);
            }
            _ => {}
        };
    }
}

impl Service for TelnetService {
    fn get_route(&self) -> &Route {
        &self.route
    }

    fn start_service(&mut self, name: Route, node: Recipient<RegisterServiceInNodeSignal>) {
        let a = self.self_addr.clone().unwrap();
        self.route = name;
        node.do_send(RegisterServiceInNodeSignal { transport: Transport::new(a.clone().recipient::<Parcel>()) });
    }

    fn can_handle_operation(&self, _route_sheet: &RouteSheet, operation: Option<String>) -> Result<bool, Error> {
        let result = match operation {
            None => false,
            Some(operation) => {
                match operation.as_str() {
                    "send_to_telnet" => true,
                    _ => false
                }
            }
        };

        Ok(result)
    }

    fn handle_message(&mut self, message: BaseMessage) {
        self.connection.write(message.data());
    }
}

impl Actor for TelnetService {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Starting! {:?}", ctx);
        self.self_addr = Some(ctx.address());


        ctx.run_interval(
            Duration::from_millis(
                self.ping_interval_in_millis.unwrap()),
            |this, _this_ctx| {
                println!("try to read");
                this.read_messages();
            },
        );
    }
}

impl Handler<Parcel> for TelnetService {
    type Result = ();

    fn handle(&mut self, msg: Parcel, _ctx: &mut Context<Self>) -> Self::Result {
        for message in msg.unpack() {
            if self.can_handle_operation(msg.route_sheet(), message.operation()).unwrap_or(false) {
                self.handle_message(message);
            }
        }
    }
}

impl Handler<GetMessagesSignal> for TelnetService {
    type Result = Result<Option<Parcel>, Error>;

    fn handle(&mut self, msg: GetMessagesSignal, _ctx: &mut Context<Self>) -> Self::Result {
        if self.messages.is_empty() {
            return Ok(None);
        };

        let messages = self.messages.clone();
        self.messages.clear();

        Ok(Some(Parcel::new(messages, RouteSheet::new(msg.send_to, self.get_route().clone()))))
    }
}

#[cfg(test)]
mod tests {
    use base::node::Node;
    use actix::{Actor, Arbiter};
    use base::route::{Route, RouteSheet};
    use crate::{TelnetService};
    use std::time::Duration;
    use base::signal::{GetMessagesSignal};
    use actix_rt::System;
    use base::message::{BaseMessage, Parcel};

    #[test]
    fn it_works() {
        System::new().block_on(async move {
            let node = Node::start(Node::new("default".to_string()));
            let arbiter = Arbiter::new();
            let telnet_service = TelnetService::start_in_arbiter(&arbiter.handle(), move |_ctx| {
                TelnetService::new(node.clone(),
                                   "185.179.2.33".to_string(), 5038, 10000000, Some(50),
                )
            });
            std::thread::sleep(Duration::from_millis(50));

            let msg = BaseMessage::new("ActionID:Login\r\nAction: Login\r\n\r\n".to_string().into_bytes(),
                                       Some("send_to_telnet".to_string()));
            telnet_service
                .send(Parcel::new(vec![msg], RouteSheet::new(
                    Route::new("telnet".to_string()), Route::new("lib".to_string())))).await.unwrap();

            let j = std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(1000));
            });

            let _r = j.join();

            let a = telnet_service.send(
                GetMessagesSignal { send_to: Route::new("telnet".to_string()) }).await.unwrap().unwrap().unwrap();

            for message in a.unpack() {
                let string = String::from_utf8(message.data().to_vec()).unwrap();
                println!("{:?}", string)
            }

            println!("{:?}", a)
        });
    }
}