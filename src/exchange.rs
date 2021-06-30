use actix::{Handler, Addr, Actor, Message, Recipient, Context, ResponseFuture};
use crate::message::{BaseSignal, BaseMessage};
use crate::source::{MessageSource, GetMessagesFromSource};
use crate::receiver::MessageReceiver;
use std::time::Duration;
use actix::dev::{ToEnvelope};
use std::fmt::Debug;

pub trait MessageExchange: Handler<BaseSignal> {}

pub struct BaseMessageExchange<MessageInType: BaseMessage + 'static,
    MessageOutType: BaseMessage + 'static,
    Source: Handler<GetMessagesFromSource<MessageInType>>,
    Receiver: Handler<MessageOutType>,
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
#[allow(dead_code)]
pub struct FanoutExchange<Msg: BaseMessage + Send + 'static> where <Msg as Message>::Result: Send {
    route: String,
    sender: Recipient<GetMessagesFromSource<Msg>>,
    receivers: Vec<Recipient<Msg>>,
    timeout: Option<Duration>,
}

impl<Msg: BaseMessage + Send + Clone + Debug + 'static> Actor for FanoutExchange<Msg> where <Msg as Message>::Result: Send {
    type Context = Context<Self>;
}

pub struct RunExchange {}

impl Message for RunExchange {
    type Result = ();
}

impl<Msg: BaseMessage + Send + Clone + Debug + 'static> Handler<RunExchange> for FanoutExchange<Msg> where <Msg as Message>::Result: Send {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, _msg: RunExchange, _ctx: &mut Context<Self>) -> Self::Result {
        let sender = self.sender.clone();
        let rcv = self.receivers.clone();
        Box::pin(async move {
            match sender.send(GetMessagesFromSource { _type: Default::default() }).await {
                Ok(messages) => {
                    for message in &messages {
                        for receiver in &rcv {
                            receiver.do_send(message.clone()).unwrap()
                        }
                    }
                }
                Err(_e) => {}
            }
        })
    }
}

impl<Msg: BaseMessage + Send + Clone + Debug + 'static> FanoutExchange<Msg> where <Msg as Message>::Result: Send {
    pub fn new(route: String, sender: Recipient<GetMessagesFromSource<Msg>>, receivers: Vec<Recipient<Msg>>) -> Self where <Msg as Message>::Result: Send {
        Self {
            route,
            sender,
            receivers,
            timeout: None,
        }
    }
    pub async fn run_once(&self) {
        match self.sender.send(GetMessagesFromSource { _type: Default::default() }).timeout(match self.timeout {
            Some(duration) => duration,
            None => Duration::new(1000000, 0)
        }).await {
            Ok(messages) => {
                for message in &messages {
                    for receiver in &self.receivers {

                        receiver.do_send(message.clone()).unwrap()
                    }
                }
            }
            Err(_e) => {}
        }
    }
}