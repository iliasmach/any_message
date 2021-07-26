use std::time::Duration;
use actix::{System, Actor, Addr, Handler, Context};
use base::core::{CoreBuilder, Core};
use base::node::Node;
use any_message_telnet::TelnetService;
use base::message::Parcel;
use base::service::{Service, ServiceCore};
use log::trace;
#[derive(Clone)]
pub struct Consumer {}

impl Consumer {}

impl Actor for Consumer {
    type Context = Context<Self>;
}

impl Service for Consumer {
    fn config_system(system_core: &mut ServiceCore, node: Addr<Node>, core: &Core) {

    }
}

impl Handler<Parcel> for Consumer {
    type Result = ();

    fn handle(&mut self, msg: Parcel, ctx: &mut Self::Context) -> Self::Result {
        trace!("{:?}", msg);
    }
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
            Node::new("telnet".to_string())
        }).await;

        let consumer = Consumer{};

        core.service("Consumer".to_string(), Box::new(Consumer::config_system));

        TelnetService::new(
            consumer.start().recipient::<Parcel>(),
            "185.179.2.33".to_string(),
            5038,
            10000000,
            Some(50)).start();



        core.run().await;
    });
}