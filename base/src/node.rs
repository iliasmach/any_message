use crate::route::{Route, Target};
use std::collections::HashMap;
use crate::transport::Transport;
use actix::{Actor, Context, Handler, AsyncContext};
use crate::message::{Parcel, Request};
use crate::signal::{RegisterServiceInNodeSignal, Heartbeat, Tick};
use log::{trace, error};
use std::time::{Duration};
use std::sync::{Arc, Mutex};
use crate::topology::Topology;

#[allow(dead_code)]
pub struct Node {
    route: Route,
    topology: Topology,
    services: HashMap<String, Transport>,
    operations: HashMap<String, Transport>,
    messages: Arc<Mutex<HashMap<Target, Vec<Parcel>>>>,
    requests: HashMap<String, Request>,
}

impl Node {
    pub fn new(node_name: String) -> Self {
        let mut route = Route::new();
        route.set_node_name(node_name);
        Self {
            route,
            topology: Topology::new(),
            services: Default::default(),
            operations: Default::default(),
            messages: Default::default(),
            requests: Default::default(),
        }
    }


    pub fn route(&self) -> &Route {
        &self.route
    }


}

impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        trace!("Starting node {}", self.route.as_string());
        ctx.notify_later(Tick::new(), Duration::from_millis(10));
    }
}

impl Handler<RegisterServiceInNodeSignal> for Node {
    type Result = ();

    fn handle(&mut self, msg: RegisterServiceInNodeSignal, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Registering service {} transport {:?}", msg.name, msg.transport);
        self.services.insert(msg.name, msg.transport);
    }
}

impl Handler<Parcel> for Node {
    type Result = ();
    #[allow(unused_must_use)]
    fn handle(&mut self, parcel: Parcel, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("Accepting parcel to {}", parcel.route_sheet().target().as_string());
        let mut messages = match self.messages.lock() {
            Ok(messages) => messages,
            Err(e) => {
                error!("Error to access messages {:?}", e);
                return;
            }
        };

        match messages.contains_key(parcel.route_sheet().target()) {
            true => {
                trace!("Saving message to node");
                match messages.get_mut(parcel.route_sheet().target()) {
                    Some(vec) => vec.push(parcel),
                    None => {}
                }
            }
            false => {
                trace!("Saving new message to node");
                messages.insert(parcel.route_sheet().target().clone(), vec![parcel]);
            }
        };
    }
}

impl Handler<Heartbeat> for Node {
    type Result = ();

    fn handle(&mut self, msg: Heartbeat, ctx: &mut Context<Self>) -> Self::Result {
        trace!("Heartbeat accepted");
    }
}


impl Handler<Tick> for Node {
    type Result = ();

    fn handle(&mut self, tick: Tick, ctx: &mut Self::Context) -> Self::Result {
        let mut messages = match self.messages.lock() {
            Ok(messages) => messages,
            Err(e) => {
                error!("Error {:?}", e);
                return;
            }
        };

        if messages.len() == 0 {}

        for (route, parcels) in messages.iter_mut() {
            match self.topology.find_transport_for_target(route) {
                Some(transport) => {
                    for parcel in parcels.drain(..) {
                        transport.send_parcel(&parcel);
                    }
                }
                None => {
                    trace!("No transport for route {}", route.as_string());
                    continue;
                }
            }
        }

        ctx.notify_later(Tick::new(), Duration::from_millis(10));
    }
}