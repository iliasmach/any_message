use crate::operation::Operation;
use crate::transport::Transport;
use std::collections::HashMap;
use crate::route::{Route, Target};
use actix::Addr;
use crate::node::Node;
use std::error::Error;
use std::fmt::{Display, Formatter};
use log::{error, trace};

pub struct Topology {
    route_table: HashMap<String, Transport>,
    subscribers: HashMap<String, Vec<Transport>>,
}

#[derive(Debug)]
pub enum TopologyError {
    RouteNotFound
}

impl Error for TopologyError {}

impl Display for TopologyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Topology {
    pub fn new() -> Self {
        Self {
            route_table: Default::default(),
            subscribers: Default::default(),
        }
    }

    pub fn add_target_transport(&mut self, route: Target, transport: Transport) {
        trace!("Adding transport for target {} {:?}", route.as_string(), transport);
        self.route_table.insert(route.as_string(), transport);
    }

    pub fn add_subscriber(&mut self, message_type: String, transport: Transport) {
        trace!("Adding subscriber for {}", message_type);
        if !self.subscribers.contains_key(&message_type) {
            self.subscribers.insert(message_type, vec![transport]);
        } else {
            match self.subscribers.get_mut(&message_type) {
                Some(transports) => {
                    transports.push(transport);
                }
                None => {
                    error!("Can`t find subscriber {}", message_type);
                }
            };
        }
    }

    pub fn route_exist(&self, route: Route) -> bool {
        self.route_table.contains_key(&route.as_string())
    }

    pub fn find_transport_for_route(&self, target: &Route) -> Option<Transport> {
        trace!("Finding transport for route {}", target.as_string());
        let string_target = target.as_string();

        match self.route_table.contains_key(string_target.as_str()) {
            false => {
                None
            }
            true => {
                let transport_ref = self.route_table.get(&string_target);
                if transport_ref.is_some() {
                    trace!("Founded!");
                    Some(transport_ref.unwrap().clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn find_consumers_for_message(&self, message_type: &String) -> Option<&Vec<Transport>> {
        match self.subscribers.contains_key(message_type) {
            false => None,
            true => {
                let transport_ref = self.subscribers.get(message_type);
                if transport_ref.is_some() {
                    Some(transport_ref.unwrap())
                } else {
                    None
                }
            }
        }
    }
}