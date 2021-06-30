use actix::{Handler, Message, Addr, Actor, Context};
use std::marker::PhantomData;
use actix::dev::{ToEnvelope};
use std::time::{Duration, Instant};
use std::sync::Mutex;

pub trait BaseMessage: Message {
    fn is_expired(&self) -> bool {
        false
    }

    fn was_in_route(&mut self, route: String);
}

/// Class for base message, that don`t have answer
#[derive(Debug, Clone)]
pub struct BaseSignal {
    data: Vec<u8>,
    ttl: Duration,
    created: Instant,
    was_in_routes: String,
}

impl BaseSignal {
    pub fn new(data: Vec<u8>, ttl: Duration) -> Self {
        Self {
            data,
            ttl,
            created: Instant::now(),
            was_in_routes: "".to_string(),
        }
    }
}

impl BaseMessage for BaseSignal {
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created) > self.ttl
    }

    fn was_in_route(&mut self, route: String) {
        if self.was_in_routes.is_empty() {
            self.was_in_routes = route;
        } else {
            self.was_in_routes = format!("{}.{}", self.was_in_routes, route);
        }
    }
}

impl Message for BaseSignal {
    type Result = ();
}

pub trait MessageSource<MessageType: Message + 'static>: Handler<GetMessagesFromSource<MessageType>> {
    fn get_messages(&mut self) -> Vec<MessageType>;
}

pub trait MessageExchange: Handler<BaseSignal> {}

pub struct BaseMessageExchange<MessageInType: BaseMessage + 'static,
    MessageOutType: BaseMessage + 'static,
    Source: MessageSource<MessageInType> + Handler<GetMessagesFromSource<MessageInType>>,
    Receiver: MessageReceiver<MessageOutType> + Handler<MessageOutType>,
    Context> {
    source: Addr<Source>,
    handler: Box<dyn Fn(MessageInType, &mut Context) -> Result<MessageOutType, Box<dyn std::error::Error>>>,
    receiver: Addr<Receiver>,
    timeout: Option<Duration>,
    context: Context,
    route: String,
}

impl<MessageInType: BaseMessage + 'static + Send,
    MessageOutType: BaseMessage + 'static + Send,
    Source: MessageSource<MessageInType> + Handler<GetMessagesFromSource<MessageInType>>,
    Receiver: MessageReceiver<MessageOutType> + Handler<MessageOutType>,
    Context> BaseMessageExchange<MessageInType, MessageOutType, Source, Receiver, Context> {
    pub fn new(route: String,
               source: Addr<Source>,
               context: Context,
               handler: Box<dyn Fn(MessageInType, &mut Context) -> Result<MessageOutType, Box<dyn std::error::Error>>>,
               receiver: Addr<Receiver>) -> Self {
        Self {
            source,
            handler,
            receiver,
            timeout: None,
            context,
            route,
        }
    }

    pub fn timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.timeout = timeout;

        self
    }

    pub async fn run(&mut self) where <Source as Actor>::Context: ToEnvelope<Source, GetMessagesFromSource<MessageInType>>,
                                      <Receiver as Actor>::Context: ToEnvelope<Receiver, MessageOutType>,
                                      <MessageOutType as Message>::Result: BaseMessage + Send
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
                        Ok(mut out_message) => {
                            out_message.was_in_route(self.route.clone());
                            self.receiver.send(out_message).await.unwrap();
                        }
                        Err(_e) => {}
                    }
                }
            }
            Err(_e) => {}
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
            if !message.is_expired() {
                vec.push(message.clone());
            }
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

    fn handle(&mut self, _msg: GetMessagesFromSource<BaseSignal>, _ctx: &mut Context<Self>) -> Self::Result {
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

    fn handle(&mut self, mut msg: BaseSignal, _ctx: &mut Context<Self>) -> Self::Result {
        msg.was_in_route("rabbitmq".to_string());
        println!("Recived message {:?}", msg)
    }
}

impl Handler<BaseSignal> for AsteriskMessageSource {
    type Result = ();

    fn handle(&mut self, mut msg: BaseSignal, _ctx: &mut Context<Self>) -> Self::Result {
        msg.was_in_route("asterisk".to_string());
        self.messages.get_mut().unwrap().push(msg);
    }
}

pub struct HandlerContext {}

#[cfg(test)]
mod tests {
    use crate::{AsteriskMessageSource, BaseMessageExchange, RabbitMessageReceiver, BaseSignal, HandlerContext};
    use actix::Actor;
    use std::sync::Mutex;
    use std::time::Duration;

    fn handle_context(message: BaseSignal, _ctx: &mut HandlerContext) -> Result<BaseSignal, Box<dyn std::error::Error>> {

        println!("Handling message {:?}", message);
        Ok(message)
    }

    #[actix_rt::test]
    async fn it_works() {
        let source = AsteriskMessageSource::start(AsteriskMessageSource { messages: Mutex::new(Vec::new()) });
        let receiver = RabbitMessageReceiver::start(RabbitMessageReceiver {});
        let mut exchange = BaseMessageExchange::new("exchange".to_string(), source.clone(), HandlerContext {}, Box::new(handle_context), receiver);
        source.send(BaseSignal::new(String::from("Привет!").into_bytes(), Duration::from_secs(5))).await.unwrap();
        exchange.run_once().await;
    }
}
