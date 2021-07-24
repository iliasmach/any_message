use crate::error::Error;
use crate::service::ServiceImpl;
use crate::node::Node;
use std::collections::HashMap;
use actix::{Addr, Arbiter, Actor, System, ArbiterHandle};
use crate::signal::Heartbeat;
use std::time::Duration;
use log::info;
use lazy_static::lazy_static;

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
    pub fn new(factory: F) -> Core {
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

    pub fn service(&self, f: Box<dyn FnOnce() -> ServiceImpl>) {
        let service = f();
    }
}