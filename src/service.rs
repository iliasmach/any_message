use crate::route::{Route};
use actix::{Recipient, Handler, Addr, Actor, Context};
use crate::signal::{Tick, LinkService};
use crate::message::{BaseMessage, Parcel, Request};
use crate::operation::{Operation};
use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};
use crate::node::{Node};
use crate::transport::Transport;
use crate::core::Core;
use actix::dev::ToEnvelope;
use log::{trace, error};
use crate::config::ServiceConfig;

pub trait Service {
    fn config_system(&mut self, system_core: &mut ServiceCore, node: Addr<Node>);
    fn handle_message(&self, message: &BaseMessage);
}

pub trait ServiceRecipient<T: Actor + Handler<Tick> + Handler<Parcel>>
    where <T as Actor>::Context: ToEnvelope<T, Tick>,
          <T as Actor>::Context: ToEnvelope<T, Parcel> {
    fn recipients(&self) -> ServiceRecipients;
}

impl<T: Actor + Handler<Tick> + Handler<Parcel>> ServiceRecipient<T> for Addr<T>
    where <T as Actor>::Context: ToEnvelope<T, Parcel>,
          <T as Actor>::Context: ToEnvelope<T, Tick>
{
    fn recipients(&self) -> ServiceRecipients
        where <T as Actor>::Context: ToEnvelope<T, Tick>,
              <T as Actor>::Context: ToEnvelope<T, Parcel>
    {
        let tick = self.clone().recipient::<Tick>();
        let parcel = self.clone().recipient::<Parcel>();
        ServiceRecipients::new(tick, parcel)
    }
}

#[derive(Debug)]
pub struct ServiceRecipients {
    tick: Recipient<Tick>,
    parcel: Recipient<Parcel>,
}

impl ServiceRecipients {
    pub fn new(tick: Recipient<Tick>,
               parcel: Recipient<Parcel>) -> Self {
        Self {
            tick,
            parcel,
        }
    }
}

pub struct ServiceFunctions {
    pub on_start: Box<dyn Fn(ServiceConfig) -> Result<Box<dyn Service>, Box<dyn std::error::Error>> + Send>,

}


pub struct ServiceCore {
    route: Route,
    operations: Vec<Operation>,
    consume_message_types: Vec<String>,
    requests_awaits: HashMap<String, Request>,
    statistics: ServiceStatistics,
    node: Addr<Node>,
    next: Option<Recipient<Parcel>>,
    recipients: Option<ServiceRecipients>,
    transport: Option<Transport>,
    functions: Option<ServiceFunctions>,
}

impl ServiceCore {
    pub fn new(service_name: String, node: Addr<Node>) -> Self {
        let route = Route::new().set_service_name(service_name).clone();
        Self {
            route,
            operations: vec![],
            consume_message_types: vec![],
            //   operation_handlers: Default::default(),
            requests_awaits: Default::default(),
            statistics: ServiceStatistics::new(),
            node,
            next: None,
            recipients: None,
            transport: None,
            functions: None,
        }
    }

    pub fn link(service: Addr<ServiceCore>, recipients: ServiceRecipients) {
        service.do_send(LinkService { recipients })
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }
    pub fn get_operations(&self) -> &Vec<Operation> {
        &self.operations
    }

    pub fn set_consuming_messages_types(&mut self, message_types: Vec<String>) {
        self.consume_message_types = message_types;
    }

    pub fn get_consuming_message_types(&self) -> Vec<String> {
        self.consume_message_types.clone()
    }

    pub fn recipients(&mut self, reciptients: ServiceRecipients) {
        self.recipients = Some(reciptients);
    }

    pub fn next(&self) -> Option<Recipient<Parcel>> {
        self.next.clone()
    }

    pub fn node(&self) -> Addr<Node> {
        self.node.clone()
    }
}

impl Actor for ServiceCore {
    type Context = Context<Self>;
}

impl Handler<Parcel> for ServiceCore {
    type Result = ();

    fn handle(&mut self, msg: Parcel, _ctx: &mut Self::Context) -> Self::Result {
        trace!("[{:?}] Consuming in system",std::thread::current().id());
        match &self.recipients {
            None => {
                self.node.do_send(msg);
            }
            Some(recepients) => {
                match recepients.parcel.do_send(msg) {
                    Err(e) => {
                        error!("Error {:?}", e);
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Handler<LinkService> for ServiceCore {
    type Result = ();

    fn handle(&mut self, msg: LinkService, _ctx: &mut Self::Context) -> Self::Result {
        self.recipients = Some(msg.recipients);
    }
}

#[derive(Debug)]
pub struct ServiceStatistics {
    start_time: NaiveDateTime,
    messages_handled: u64,
    requests_handled: u64,
    messages_per_second: u64,
    requests_per_second: u64,
}

impl ServiceStatistics {
    pub fn new() -> Self {
        Self {
            start_time: Utc::now().naive_local(),
            messages_handled: 0,
            requests_handled: 0,
            messages_per_second: 0,
            requests_per_second: 0,
        }
    }
}