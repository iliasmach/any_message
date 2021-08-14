use std::time::Duration;
use actix::{System, Actor, Addr, Handler, Context};
use base::core::{CoreBuilder, Core};
use base::node::Node;
use any_message_telnet::TelnetService;
use base::message::Parcel;
use base::service::{Service, ServiceCore, ServiceRecipients, ServiceRecipient};
use log::trace;
use base::signal::{Tick};
use base::transport::Transport;

#[derive(Clone)]
pub struct Consumer {}

impl Consumer {}

impl Actor for Consumer {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("Consumer started!");
    }
}

impl Service for Consumer {
    fn config_system(service_code: &mut ServiceCore, node: Addr<Node>, core: &Core) {
        service_code.set_consuming_messages_types(vec!["Asterisk Message".to_string()]);
    }
}

impl Handler<Parcel> for Consumer {
    type Result = ();

    fn handle(&mut self, msg: Parcel, ctx: &mut Self::Context) -> Self::Result {

        trace!("{:?} Consuming in comsumer {:?}", std::thread::current().id(), msg);
    }
}

impl Handler<Tick> for Consumer {
    type Result = ();

    fn handle(&mut self, msg: Tick, ctx: &mut Self::Context) -> Self::Result {}
}

fn main() {
    dotenv::dotenv();
    env_logger::init();
    let j = std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(5));
        std::process::exit(0);
    });
    System::new().block_on(async move {
        let mut core = CoreBuilder::new(|| {
            let node = Node::new("telnet".to_string());

            node
        }).await;

        let service = core.service("Consumer".to_string(), Box::new(Consumer::config_system)).await;
        let telnet_service = core.service("Asterisk".to_string(), Box::new(TelnetService::config_system)).await;

        let telnet_actor = TelnetService::new(
            core.node().clone().recipient::<Parcel>(),
            "Asterisk Message".to_string(),
            "185.179.2.33".to_string(),
            5038,
            10000000,
            Some(50)).start();

        let consumer_actor = Consumer::start(Consumer{});

        ServiceCore::link(telnet_service, telnet_actor.recipients());
        ServiceCore::link(service, consumer_actor.recipients());



        core.run().await;
    });
}