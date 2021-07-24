use crate::route::{Route, RouteSheet};
use actix::{Recipient, Handler, Addr};
use crate::signal::{RegisterServiceInNodeSignal, GetMessagesSignal};
use crate::error::Error;
use crate::message::{BaseMessage, Parcel, Request};
use crate::operation::Operation;
use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};
use crate::node::Node;
use semver::Version;

pub trait Service: Handler<Parcel> {
    fn get_route(&self) -> &Route;
    fn start_service(&mut self, name: Route, node: Recipient<RegisterServiceInNodeSignal>);
    fn can_handle_operation(&self, route_sheet: &RouteSheet, operation: Option<String>) -> Result<bool, Error>;
    fn handle_message(&mut self, message: BaseMessage);
}

pub struct ServiceImpl {
    route: Route,
    operations: Vec<Operation>,
    requests_awaits: HashMap<String, Request>,
    statistics: ServiceStatistics,
    node: Option<Addr<Node>>,
}

impl ServiceImpl {
    pub fn new(route: Route) -> Self {
        Self {
            route,
            operations: vec![],
            requests_awaits: Default::default(),
            statistics: ServiceStatistics::new(),
            node: None
        }
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