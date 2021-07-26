use crate::route::{Route, RouteSheet};
use actix::{Recipient, Handler, Addr, Actor, Context};
use crate::signal::{RegisterServiceInNodeSignal, GetMessagesSignal, Tick};
use crate::error::Error;
use crate::message::{BaseMessage, Parcel, Request};
use crate::operation::{Operation, OperationHandler};
use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};
use crate::node::{Node};
use semver::Version;
use crate::transport::Transport;
use crate::core::Core;
use actix::dev::ToEnvelope;

pub trait Service {
    fn config_system(system_core: &mut ServiceCore, node: Addr<Node>, core: &Core) where Self: Sized;
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

pub struct ServiceCore {
    route: Route,
    operations: Vec<Operation>,
    operation_handlers: HashMap<Operation, Box<dyn Fn(&BaseMessage) -> Result<(), Error> + Send>>,
    requests_awaits: HashMap<String, Request>,
    statistics: ServiceStatistics,
    node: Addr<Node>,
    next: Option<Recipient<Parcel>>,
    recipients: Option<ServiceRecipients>,
    transport: Option<Transport>,
}

impl ServiceCore {
    pub fn new(service_name: String, node: Addr<Node>) -> Self {
        let route = Route::new().set_service_name(service_name).clone();
        Self {
            route,
            operations: vec![],
            operation_handlers: Default::default(),
            requests_awaits: Default::default(),
            statistics: ServiceStatistics::new(),
            node,
            next: None,
            recipients: None,
            transport: None,
        }
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
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

    pub fn operation_handler<F>(&mut self, operation: Operation, handler: F)
        where F: Fn(&BaseMessage) -> Result<(), Error> + Clone + 'static + Send
    {
        self.operation_handlers.insert(operation, Box::new(handler));
    }
}

impl Actor for ServiceCore {
    type Context = Context<Self>;
}

impl Handler<Parcel> for ServiceCore {
    type Result = ();

    fn handle(&mut self, msg: Parcel, ctx: &mut Self::Context) -> Self::Result {}
}

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