extern crate log;


use base::signal::{GetMessagesSignal, RegisterServiceInNodeSignal, Tick};
use base::error::Error;
use base::message::{Parcel, BaseMessage};
use telnet::{Telnet, TelnetEvent};
use base::route::{RouteSheet, Route, Target};
use log::{debug, info, trace};
use base::service::{ServiceCore, Service};
use base::transport::Transport;
use std::time::Duration;
use base::node::{Node};
use actix::prelude::*;
use base::core::Core;
use base::operation::Operation;
use semver::Version;

pub struct TelnetService {
    host: String,
    port: u16,
    buff_size: u32,
    connection: Telnet,
    buf: String,
    ping_interval_in_millis: Option<u64>,
    messages: Vec<BaseMessage>,
    send_to: Recipient<Parcel>,
    message_type: String,
}


impl TelnetService {
    pub fn new(send_to: Recipient<Parcel>,
               message_type: String,
               host: String,
               port: u16,
               buff_size: u32,
               ping_interval_in_millis: Option<u64>) -> Self {
        info!("Connection to {}:{}", host, port);

        let connection = match Telnet::connect((host.clone().as_str(), port.clone()), buff_size.clone() as usize) {
            Ok(conn) => conn,
            Err(_e) => {
                panic!("Error while connecting to asterisk {:?}", _e);
            }
        };

        let this = Self {
            host,
            port,
            buff_size,
            connection,
            buf: String::with_capacity(buff_size as usize),
            ping_interval_in_millis,
            messages: vec![],
            send_to,
            message_type
        };

        this
    }

    pub fn message_type(&mut self, message_type: String) -> &mut Self {
        self.message_type = message_type;

        self
    }

    pub fn read_messages(&mut self) {
        let event = match self.connection.read_timeout(Duration::from_millis(50)) {
            Ok(event) => event,
            Err(_e) => {
                debug!("Read 2 secs");
                return;
            }
        };

        match event {
            TelnetEvent::Data(buffer) => {
                debug!("{:?}", String::from_utf8(buffer.clone().to_vec()));
                let message = BaseMessage::new(buffer.to_vec(), None);

                self.messages.push(message);
            }
            _ => {}
        };
    }

    pub fn send_messages(&mut self) {
        if self.messages.is_empty() {
            return;
        }
        trace!("Try to send");
        let messages = self.messages.clone();
        self.messages.clear();

        trace!("Sending messages");
        self.send_to.do_send(
            Parcel::new(messages, RouteSheet::new(
                Target::Consumer(self.message_type.clone()), Route::new(),
            )));
    }
}

impl Actor for TelnetService {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Starting! {:?}", ctx);
        // self.self_addr = Some(ctx.address());

        ctx.run_interval(
            Duration::from_millis(
                self.ping_interval_in_millis.unwrap()),
            |this, _this_ctx| {
                this.read_messages();
            },
        );

        ctx.run_interval(
            Duration::from_millis(
                100),
            |this, _this_ctx| {
                this.send_messages();
            },
        );
    }
}

impl Service for TelnetService {
    fn config_system(service_core: &mut ServiceCore, node: Addr<Node>, core: &Core) where Self: Sized {
        service_core.add_operation(
            Operation::new(
                "SendMessageToTelnet".to_string(),
                Version::new(1, 0,0),
                "".to_string()
            )
        );

        service_core.set_consuming_messages_types(vec!["TelnetCommand".to_string()]);
    }
}

impl Handler<Parcel> for TelnetService {
    type Result = ();

    fn handle(&mut self, msg: Parcel, ctx: &mut Self::Context) -> Self::Result {
        trace!("Consuming message in telnet {:?}", msg);
    }
}

impl Handler<Tick> for TelnetService {
    type Result = ();

    fn handle(&mut self, msg: Tick, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use base::node::{Node};
    use actix::{Actor, Arbiter};
    use base::route::{Route, RouteSheet};
    use crate::{TelnetService};
    use std::time::Duration;
    use base::signal::{GetMessagesSignal, Tick};
    use actix_rt::System;
    use base::message::{BaseMessage, Parcel};
    use base::core::{Core, CoreBuilder};
    use base::service::{ServiceCore, Service, ServiceRecipients, ServiceRecipient};
    use log::info;
    use base::operation::{OperationHandler, Operation};
    use semver::Version;

    #[test]
    fn it_works() {
        dotenv::dotenv();
        env_logger::init();
        let j = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(5));
            std::process::exit(0);
        });
        System::new().block_on(async move {
            let core = CoreBuilder::new(|| {
                Node::new("telnet".to_string())
            }).await;


            let telnet = TelnetService::new(
                core.node().recipient::<Parcel>(),
                "TelnetMesage".to_string(),
                "185.179.2.33".to_string(),
                5038,
                10000000,
                Some(50));

            let telnet_service = TelnetService::start(telnet);


            core.service(
                "telnet".to_string(),
                telnet_service.recipients(),
                ).await;

            core.run().await;
        });
    }
}