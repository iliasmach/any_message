use actix::{Handler, Message, Addr, Actor, ActorContext, Context, AsyncContext};
use std::collections::HashMap;
use std::marker::PhantomData;
use actix::dev::{ToEnvelope, MessageResponse};
use std::time::Duration;
use std::sync::Mutex;

pub trait BaseMessage {}

/// Class for base message, that don`t have answer
#[derive(Debug, Clone)]
pub struct BaseSignal {
    data: Vec<u8>,
}

impl BaseSignal {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data
        }
    }
}

impl BaseMessage for BaseSignal {}

impl Message for BaseSignal {
    type Result = ();
}

pub trait MessageSource<MessageType: Message + 'static>: Handler<GetMessagesFromSource<MessageType>> {
    fn get_messages(&mut self) -> Vec<MessageType>;
}

pub trait MessageExchange: Handler<BaseSignal> {}

pub struct BaseMessageExchange<MessageInType: Message + 'static,
    MessageOutType: Message + 'static,
    Source: MessageSource<MessageInType> + Handler<GetMessagesFromSource<MessageInType>>,
    Receiver: MessageReceiver<MessageOutType> + Handler<MessageOutType>,
    Context> {
    source: Addr<Source>,
    handler: Box<dyn Fn(MessageInType, &mut Context) -> Result<MessageOutType, Box<dyn std::error::Error>>>,
    receiver: Addr<Receiver>,
    timeout: Option<Duration>,
    context: Context
}

impl<MessageInType: Message + 'static + Send,
    MessageOutType: Message + 'static + Send,
    Source: MessageSource<MessageInType> + Handler<GetMessagesFromSource<MessageInType>>,
    Receiver: MessageReceiver<MessageOutType> + Handler<MessageOutType>,
    Context> BaseMessageExchange<MessageInType, MessageOutType, Source, Receiver, Context> {
    pub fn new(source: Addr<Source>,
               context: Context,
               handler: Box<dyn Fn(MessageInType, &mut Context) -> Result<MessageOutType, Box<dyn std::error::Error>>>,
               receiver: Addr<Receiver>) -> Self {
        Self {
            source,
            handler,
            receiver,
            timeout: None,
            context
        }
    }

    pub fn timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.timeout = timeout;

        self
    }

    pub async fn run(&mut self) where <Source as Actor>::Context: ToEnvelope<Source, GetMessagesFromSource<MessageInType>>,
                                  <Receiver as Actor>::Context: ToEnvelope<Receiver, MessageOutType>,
                                  <MessageOutType as Message>::Result: Send
    {
        loop {
            self.run_once().await
        }
    }

    pub async fn run_once(&mut self) where <Source as Actor>::Context: ToEnvelope<Source, GetMessagesFromSource<MessageInType>>,
                                       <Receiver as Actor>::Context: ToEnvelope<Receiver, MessageOutType>,
                                       <MessageOutType as Message>::Result: Send {
        match self.source.send(GetMessagesFromSource { _type: Default::default() })
            .timeout(match self.timeout {
                Some(duration) => duration,
                None => Duration::new(1000000, 0)
            }).await {
            Ok(messages) => {
                for message in messages {
                    match (self.handler)(message, &mut self.context) {
                        Ok(out_message) => {
                            self.receiver.send(out_message).await;
                        }
                        Err(e) => {}
                    }
                }
            }
            Err(e) => {}
        }
    }
}

pub struct GetMessagesFromSource<MessageType: Message> {
    _type: PhantomData<MessageType>,
}

impl<MessageType: Message + 'static> Message for GetMessagesFromSource<MessageType> {
    type Result = Vec<MessageType>;
}

pub trait MessageReceiver<MessageType: Message> {}

pub struct AsteriskMessageSource {
    messages: Mutex<Vec<BaseSignal>>,
}

impl MessageSource<BaseSignal> for AsteriskMessageSource {
    fn get_messages(&mut self) -> Vec<BaseSignal> {
        let self_messages = self.messages.get_mut().unwrap();

        let mut vec: Vec<BaseSignal> = Vec::with_capacity(self_messages.len());

        for message in self_messages.into_iter() {
            vec.push(message.clone());
        }

        self_messages.clear();

        vec
    }
}

impl Actor for AsteriskMessageSource {
    type Context = Context<Self>;
}

impl Handler<GetMessagesFromSource<BaseSignal>> for AsteriskMessageSource {
    type Result = Vec<BaseSignal>;

    fn handle(&mut self, msg: GetMessagesFromSource<BaseSignal>, ctx: &mut Context<Self>) -> Self::Result {
        self.get_messages()
    }
}

pub struct RabbitMessageReceiver {}

impl Actor for RabbitMessageReceiver {
    type Context = Context<Self>;
}

impl MessageReceiver<BaseSignal> for RabbitMessageReceiver {}

impl Handler<BaseSignal> for RabbitMessageReceiver {
    type Result = ();

    fn handle(&mut self, msg: BaseSignal, ctx: &mut Context<Self>) -> Self::Result {
        println!("Recived message {:?}", msg.data)
    }
}

impl Handler<BaseSignal> for AsteriskMessageSource {
    type Result = ();

    fn handle(&mut self, msg: BaseSignal, ctx: &mut Context<Self>) -> Self::Result {
        self.messages.get_mut().unwrap().push(msg);
    }
}

pub struct HandlerContext {}

#[cfg(test)]
mod tests {
    use crate::{AsteriskMessageSource, BaseMessageExchange, RabbitMessageReceiver, BaseSignal, HandlerContext};
    use actix::Actor;
    use std::sync::Mutex;

    fn handle_context(message: BaseSignal, ctx: &mut HandlerContext) -> Result<BaseSignal, Box<dyn std::error::Error>> {
        println!("Handling message {:?}", message);
        Ok(message)
    }

    #[actix_rt::test]
    async fn it_works() {
        let source = AsteriskMessageSource::start(AsteriskMessageSource { messages: Mutex::new(Vec::new()) });
        let receiver = RabbitMessageReceiver::start(RabbitMessageReceiver {});
        let mut exchange = BaseMessageExchange::new(source.clone(), HandlerContext {}, Box::new(handle_context), receiver  );
        source.send(BaseSignal { data: String::from("Привет!").into_bytes() }).await;
        exchange.run_once().await;
    }
}
