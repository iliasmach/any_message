extern crate log;

use base::signal::{GetMessagesSignal, RegisterServiceInNodeSignal, Tick};
use base::error::Error;
use base::message::{Parcel, BaseMessage};
use telnet::{Telnet, TelnetEvent};
use base::route::{RouteSheet, Route};
use log::{debug, info, trace};
use base::service::{ServiceCore};
use base::transport::Transport;
use std::time::Duration;
use base::node::{Node};
use actix::prelude::*;

pub struct TelnetService {
    host: String,
    port: u16,
    buff_size: u32,
    connection: Telnet,
    buf: String,
    ping_interval_in_millis: Option<u64>,
    service_core: Recipient<Parcel>,
    messages: Vec<BaseMessage>,
}


impl TelnetService {
    pub fn new(recipient: Recipient<Parcel>, host: String, port: u16, buff_size: u32, ping_interval_in_millis: Option<u64>) -> Self {
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
            service_core: recipient,
            messages: vec![],
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
                debug!("{:?}", String::from_utf8(buffer.clone().to_vec()));
                let message = BaseMessage::new(buffer.to_vec(), None);

                self.messages.push(message);
            }
            _ => {}
        };
    }

    pub fn send_messages(&mut self) {
        let messages = self.messages.clone();
        self.messages.clear();
        let service = self.service_core.clone();
        trace!("Sending messages");
        service.do_send(
            Parcel::new(messages, RouteSheet::new(
                Route::new(), Route::new(),
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
                info!("try to read");
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
    use base::service::ServiceCore;
    use log::info;

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

            let telnet = TelnetService::new(core.node().recipient::<Parcel>(), "185.179.2.33".to_string(),
                                            5038,
                                            10000000,
                                            Some(50));

            let telnet_service = TelnetService::start(telnet);

            core.service("telnet".to_string(), None, move |mut service_core, node, core| {
                info!("Registering Telnet");

                service_core
            }).await;

            core.run().await;
        });
    }
}