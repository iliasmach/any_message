extern crate log;
#[macro_use]
extern crate base;

use base::message::{BaseMessage, Parcel};
use telnet::{Telnet, TelnetEvent};
use actix::{Recipient, Context, Actor, AsyncContext, Addr, Handler};
use std::time::Duration;
use base::route::{RouteSheet, Route, Target};
use base::service::{Service, ServiceCore, ServiceFunctions};
use base::node::Node;
use base::core::Core;
use base::operation::Operation;
use semver::Version;
use base::signal::Tick;
use log::{info, debug, trace};
use base::plugin::Plugin;
use base::config::ServiceConfig;

pub struct TelnetService {
    host: String,
    port: u16,
    buff_size: u32,
    connection: Telnet,
    buf: String,
    ping_interval_in_millis: Option<u64>,
    messages: Vec<BaseMessage>,
    message_type: String,
}


impl TelnetService {
    pub fn on_start(config: ServiceConfig) -> Result<Box<dyn Service>, Box<dyn std::error::Error>> {
        println!("{:?}", config);
        let message_type = config.parameters.get("message_type").unwrap();
        let host = config.parameters.get("host").unwrap();
        let port : u16= config.parameters.get("port").unwrap().parse().unwrap();
        let buffer_size: u32 = config.parameters.get("buffer_size").unwrap().parse().unwrap();
        let delay_in_millis: u64 = config.parameters.get("delay_in_millis").unwrap().parse().unwrap();
        let telnet_service=  TelnetService::new(
            message_type.clone(),
            host.clone(),
            port as u16,
            buffer_size as u32,
            Some(delay_in_millis as u64)
        );
        Ok(Box::new(telnet_service))
    }

    pub fn new(message_type: String,
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
            message_type,
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
}

#[no_mangle]
pub extern "C" fn serv_config(config: &ServiceConfig) -> Result<ServiceFunctions, Box<dyn std::error::Error>> {
    Ok(ServiceFunctions {
        on_start: Box::new(TelnetService::on_start)
    })
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

        // ctx.run_interval(
        //     Duration::from_millis(
        //         100),
        //     |this, _this_ctx| {
        //         this.send_messages();
        //     },
        // );
    }
}

impl Service for TelnetService {
    fn config_system(service_core: &mut ServiceCore, node: Addr<Node>, core: &Core) where Self: Sized {
        service_core.add_operation(
            Operation::new(
                "SendMessageToTelnet".to_string(),
                Version::new(1, 0, 0),
                "".to_string(),
            )
        );

        service_core.set_consuming_messages_types(vec!["TelnetCommand".to_string()]);
    }

    fn handle_message(&self, message: &BaseMessage) {
        trace!("Consuming message in telnet {:?}", message);
    }
}

impl Handler<Parcel> for TelnetService {
    type Result = ();

    fn handle(&mut self, msg: Parcel, ctx: &mut Self::Context) -> Self::Result {
        trace!("Consuming message in telnet {:?}", msg);

        let messages = msg.unpack();

        for message in messages {
            self.handle_message(message);
        }
    }
}

impl Handler<Tick> for TelnetService {
    type Result = ();

    fn handle(&mut self, msg: Tick, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

#[derive(Default)]
pub struct TelnetServicePlugin;

impl Plugin for TelnetServicePlugin {
    fn name(&self) -> &'static str {
        "TelnetServicePlugin"
    }

    fn on_load(&self, core: &mut Core) {
        debug!("Loading telnet plugin");
        core.service_config("TelnetService".to_string(), Box::new(serv_config));
    }

    fn on_unload(&self) {
        info!("Telnet plugin unloaded");
    }
}

declare_plugin!(TelnetServicePlugin, TelnetServicePlugin::default);

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
        System::new().block_on(
            async move {
                let core =
                    CoreBuilder::new(|| {
                        Node::new("telnet".to_string())
                    }).plugins(vec!["target/debug/libany_message_telnet.rlib".to_string()]).build();


                let telnet = TelnetService::new(
                    "TelnetMesage".to_string(),
                    "185.179.2.33".to_string(),
                    5038,
                    10000000,
                    Some(50));

                let telnet_service = TelnetService::start(telnet);


                core.service(
                    "telnet".to_string(),
                    TelnetService::config_system,
                ).await;

                core.run().await;
            }
        );
    }
}