use crate::message::{BaseSignal, BaseMessage};
use actix::{Context, Actor, Handler};
use crate::receiver::MessageReceiver;

pub struct RabbitMessageReceiver {
    pub(crate) last_message: Option<BaseSignal>
}

impl RabbitMessageReceiver {
    pub fn last_message(&self) -> Option<BaseSignal> {
        self.last_message.clone()
    }
}

impl Actor for RabbitMessageReceiver {
    type Context = Context<Self>;
}

impl MessageReceiver<BaseSignal> for RabbitMessageReceiver {}

impl Handler<BaseSignal> for RabbitMessageReceiver {
    type Result = ();

    fn handle(&mut self, mut msg: BaseSignal, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Recived message {:?}", msg);
        msg.was_in_route("rabbitmq".to_string());
        self.last_message = Some(msg.clone());
        // std::thread::sleep(Duration::from_secs(2));

    }
}