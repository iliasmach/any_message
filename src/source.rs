use actix::{Message};
use std::marker::PhantomData;
use crate::message::BaseMessage;

pub trait MessageSource<MessageType: BaseMessage + 'static> {
    fn new(route: String ) -> Self;
    fn get_messages(&mut self) -> Vec<MessageType>;
}

pub struct GetMessagesFromSource<MessageType: BaseMessage> {
    pub(crate) _type: PhantomData<MessageType>,
}

impl<MessageType: BaseMessage + 'static> Message for GetMessagesFromSource<MessageType> {
    type Result = Vec<MessageType>;
}