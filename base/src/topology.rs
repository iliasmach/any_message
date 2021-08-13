use crate::operation::Operation;
use crate::transport::Transport;
use std::collections::HashMap;
use crate::route::{Route, Target};
use actix::Addr;
use crate::node::Node;
use std::error::Error;
use std::fmt::{Display, Formatter};
use log::trace;

pub struct Topology {
    route_table: HashMap<String, Transport>,
}
#[derive(Debug)]
pub enum TopologyError {
    RouteNotFound
}

impl Error for TopologyError {

}

impl Display for TopologyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Topology {
    pub fn new() -> Self {
        Self {
            route_table: Default::default(),
        }
    }

    pub fn add_target_transport(&mut self, route: Target, transport: Transport) {
        self.route_table.insert(route.as_string(), transport);
    }

    pub fn route_exist(&self, route: Route) -> bool {
        self.route_table.contains_key(&route.as_string())
    }

    pub fn find_transport_for_target(&self, target: &Target) -> Option<Transport> {
        trace!("Finding transport for route {}", target.as_string());
        let string_target = target.as_string();

        match self.route_table.contains_key(&*string_target) {
            false => {
                None
            }
            true => {
                let transport_ref = self.route_table.get(&string_target);
                if transport_ref.is_some() {
                    Some(transport_ref.unwrap().clone())
                } else {
                    None
                }
            }
        }
    }

}