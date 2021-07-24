use crate::error::Error;
use crate::node::Node;
use std::collections::HashMap;
use actix::{Addr, Arbiter, Actor, System, ArbiterHandle, Recipient};
use crate::signal::{Heartbeat, RegisterServiceInNodeSignal};
use std::time::Duration;
use log::{info, trace};
use lazy_static::lazy_static;
use crate::service::{ServiceCore};
use crate::transport::Transport;
use crate::message::Parcel;

pub struct CoreBuilder<F>
    where
        F: Fn() -> Node
{
    factory: F,
}

impl<F> CoreBuilder<F>
    where
        F: Fn() -> Node
{
    pub async fn new(factory: F) -> Core {
        let node = factory();

        let arbiter = Arbiter::new().handle();
        let node = Node::start_in_arbiter(&arbiter, |ctx| {
            node
        });

        Core {
            arbiter,
            node,
        }
    }
}

pub struct Core {
    arbiter: ArbiterHandle,
    node: Addr<Node>,
}

impl Core {
    pub fn new(name: String) -> Self {
        info!("AnyMessage core started!");
        let arbiter = Arbiter::new().handle();
        let node = Node::start_in_arbiter(&arbiter, |ctx| {
            Node::new(name)
        });

        Core {
            arbiter,
            node,
        }
    }

    pub async fn run(&self) -> Result<(), Error> {
        loop {
            match self.node.send(Heartbeat {}).await {
                Ok(()) => {}
                Err(_e) => {}
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        Ok(())
    }

    pub fn node(&self) -> Addr<Node> {
        self.node.clone()
    }

    pub async fn service<F:Fn(ServiceCore, Addr<Node>, &Core)->ServiceCore>(&self, service_name:String, next: Option<Recipient<Parcel>>, mut config: F) -> Addr<ServiceCore> {
        let service_core = config(ServiceCore::new(service_name.clone(), self.node.clone(), next), self.node.clone(), self);

        let arbiter = Arbiter::new().handle();

        let service_addr = ServiceCore::start_in_arbiter(&arbiter, |ctx| {
            service_core
        });
        
        self.node.send(RegisterServiceInNodeSignal {
            transport: Transport::new(service_addr.clone().recipient::<Parcel>()),
            name: service_name,
        });

        service_addr
    }
}