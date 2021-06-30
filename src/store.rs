use actix::Handler;
use crate::message::BaseMessage;

pub trait MessageStore<MessageType: BaseMessage> : Handler<MessageType> {

}