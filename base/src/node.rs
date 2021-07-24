use crate::route::Route;
use std::collections::HashMap;
use crate::transport::Transport;
use actix::{Actor, Context, Handler, AsyncContext, Message};
use crate::message::{Parcel, Request};
use crate::signal::{RegisterServiceInNodeSignal, Heartbeat};
use crate::service::ServiceImpl;
use log::{trace, error};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub struct Node {
    route: Route,
    node_addresses: HashMap<String, Transport>,
    services: HashMap<String, Transport>,
    operations: HashMap<String, Transport>,
    messages: Arc<Mutex<HashMap<Route, Vec<Parcel>>>>,
    requests: HashMap<String, Request>,
}

impl Node {
    pub fn new(node_name: String) -> Self {
        let mut route = Route::new();
        route.set_node_name(node_name);
        Self {
            route,
            node_addresses: Default::default(),
            services: Default::default(),
            operations: Default::default(),
            messages: Default::default(),
            requests: Default::default(),
        }
    }


    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn has_route(&self, route: Route) -> bool {
        let target = route.as_string();
        match self.services.contains_key(&*target) {
            false => {
                match self.node_addresses.contains_key(&*target) {
                    true => {
                        true
                    }
                    false => false
                }
            }
            true => {
                let transport_ref = self.services.get(&*target);
                if transport_ref.is_some() {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn find_transport_for_route(&self, route: &Route) -> Option<Transport> {
        trace!("Finding transport for route {}", route.as_string());
        let target = route.as_string();

        match self.services.contains_key(&*target) {
            false => {
                match self.node_addresses.contains_key(&*target) {
                    true => {
                        let transport_ref = self.node_addresses.get(&*target);
                        if transport_ref.is_some() {
                            Some(transport_ref.unwrap().clone())
                        } else {
                            None
                        }
                    }
                    false => None
                }
            }
            true => {
                let transport_ref = self.services.get(&target);
                if transport_ref.is_some() {
                    Some(transport_ref.unwrap().clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn add_service(&mut self, service: ServiceImpl) {

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
                error!("Error to access messages");
                return;
            }
        };

        match messages.contains_key(parcel.route_sheet().target()) {
            true => {
                match messages.get_mut(parcel.route_sheet().target()) {
                    Some(vec) => vec.push(parcel),
                    None => {}
                }
            }
            false => {
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

struct Tick {
    time: Instant,
}

impl Tick {
    pub fn new() -> Self {
        Self {
            time: Instant::now()
        }
    }
}

impl Message for Tick {
    type Result = ();
}

impl Handler<Tick> for Node {
    type Result = ();

    fn handle(&mut self, tick: Tick, ctx: &mut Self::Context) -> Self::Result {
        trace!("Tick {}", tick.time.elapsed().as_millis());
        let mut messages = match self.messages.lock() {
            Ok(messages) => messages,
            Err(e) => {
                error!("Error {:?}", e);
                return;
            }
        };

        if messages.len() == 0 {
            trace!("No parcels");
        }

        for (route, parcels) in messages.iter_mut() {
            match self.find_transport_for_route(route) {
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