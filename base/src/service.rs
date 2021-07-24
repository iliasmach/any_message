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

pub struct ServiceRecipients {
    tick: Option<Recipient<Tick>>,
    parcel: Option<Recipient<Parcel>>,
}

pub struct ServiceCore {
    route: Route,
    operations: Vec<Operation>,
    operation_handlers: HashMap<Operation, OperationHandler>,
    requests_awaits: HashMap<String, Request>,
    statistics: ServiceStatistics,
    node: Addr<Node>,
    next: Option<Recipient<Parcel>>,
    recipients: ServiceRecipients,
    transport: Option<Transport>
}

impl ServiceCore {
    pub fn new(service_name: String, node: Addr<Node>, next: Option<Recipient<Parcel>>) -> Self {
        let route= Route::new().set_service_name(service_name).clone();
        Self {
            route,
            operations: vec![],
            operation_handlers: Default::default(),
            requests_awaits: Default::default(),
            statistics: ServiceStatistics::new(),
            node,
            next,
            recipients: ServiceRecipients { tick: None, parcel: None },
            transport: None
        }
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    pub fn tick_recipient(&mut self, recipient: Recipient<Tick>) {
        self.recipients.tick = Some(recipient);
    }
}

impl Actor for ServiceCore {
    type Context = Context<Self>;
}

impl Handler<Parcel> for ServiceCore {
    type Result = ();

    fn handle(&mut self, msg: Parcel, ctx: &mut Self::Context) -> Self::Result {

    }
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