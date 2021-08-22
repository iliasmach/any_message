use std::time::Duration;
use actix::{System, Actor};
use base::core::CoreBuilder;
use base::node::Node;
use base::service::{ServiceRecipient, Service};
use any_message_telnet::TelnetService;
use telnet::Telnet;
use base::message::Parcel;


fn main() {
    dotenv::dotenv();
    env_logger::init();
    let j = std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(5));
        std::process::exit(0);
    });
    System::new().block_on(async move {
        let core = unsafe {
            CoreBuilder::new(|| {
                Node::new("telnet".to_string())
            }).build()
        };


        TelnetService::new(
            core.node().recipient::<Parcel>(),
            "185.179.2.33".to_string(),
            5038,
            10000000,
            Some(50)).start();



        core.run().await;
    });
}