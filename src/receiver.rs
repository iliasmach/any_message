use actix::Message;

pub trait MessageReceiver<MessageType: Message> {}