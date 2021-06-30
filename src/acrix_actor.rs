use actix::{Actor, Context, Handler};
use crate::source::{GetMessagesFromSource, MessageSource};
use crate::message::{BaseSignal, BaseMessage};
use std::sync::Mutex;

impl Actor for ActorMessageSource<BaseSignal> {
    type Context = Context<Self>;
}

impl Handler<GetMessagesFromSource<BaseSignal>> for ActorMessageSource<BaseSignal> {
    type Result = Vec<BaseSignal>;

    fn handle(&mut self, _msg: GetMessagesFromSource<BaseSignal>, _ctx: &mut Context<Self>) -> Self::Result {
        self.get_messages()
    }
}


pub struct ActorMessageSource<MessageType: BaseMessage> {
    messages: Mutex<Vec<MessageType>>,
    route: String,
}

impl MessageSource<BaseSignal> for ActorMessageSource<BaseSignal> {
    fn new(route: String) -> Self {
        Self {

            messages: Mutex::new(vec![]),
            route
        }
    }

    fn get_messages(&mut self) -> Vec<BaseSignal> {
        let self_messages = self.messages.get_mut().unwrap();

        let mut vec: Vec<BaseSignal> = Vec::with_capacity(self_messages.len());

        for message in self_messages.into_iter() {
            if !message.is_expired() {
                vec.push(message.clone());
            }
        }

        self_messages.clear();

        vec
    }
}

impl Handler<BaseSignal> for ActorMessageSource<BaseSignal> {
    type Result = ();

    fn handle(&mut self, mut msg: BaseSignal, _ctx: &mut Context<Self>) -> Self::Result {
        msg.was_in_route(self.route.clone());
        self.messages.get_mut().unwrap().push(msg);
    }
}