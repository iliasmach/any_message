use crate::route::Route;
use std::collections::HashMap;
use crate::transport::Transport;
use actix::{Actor, Context, Handler, ResponseFuture};
use crate::message::Parcel;
use crate::signal::RegisterServiceInNodeSignal;

#[allow(dead_code)]
pub struct Node {
    route: Route,
    node_addresses: HashMap<String, Transport>,
    services: HashMap<String, Transport>,
}

impl Node {
    pub fn new(name: String) -> Self {
        let route = Route::new(name);
        Self {
            route,
            node_addresses: Default::default(),
            services: Default::default()
        }
    }

    pub fn has_route(&self, route: Route) -> bool {
        let target = route.name();
        match self.services.contains_key(target) {
            false => {
                match self.node_addresses.contains_key(target) {
                    true => {
                        true
                    }
                    false => false
                }
            }
            true => {
                let transport_ref = self.services.get(target);
                if transport_ref.is_some() {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn find_transport_for_route(&self, route: Route) -> Option<Transport> {
        let target = route.name();

        match self.services.contains_key(target) {
            false => {
                match self.node_addresses.contains_key(target) {
                    true => {
                        let transport_ref = self.node_addresses.get(target);
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
                let transport_ref = self.services.get(target);
                if transport_ref.is_some() {
                    Some(transport_ref.unwrap().clone())
                } else {
                    None
                }
            }
        }
    }
}

impl Actor for Node {
    type Context = Context<Self>;
}

impl Handler<RegisterServiceInNodeSignal> for Node {
    type Result = ();

    fn handle(&mut self, _msg: RegisterServiceInNodeSignal, _ctx: &mut Context<Self>) -> Self::Result {

    }
}

impl Handler<Parcel> for Node {
    type Result = ResponseFuture<()>;
    #[allow(unused_must_use)]
    fn handle(&mut self, msg: Parcel, _ctx: &mut Context<Self>) -> Self::Result {
        let transport = self.find_transport_for_route(msg.target().clone());
        Box::pin(async move {
            if transport.is_some() {
                transport.unwrap().send_parcel(msg.clone());
            }
        })
    }
}