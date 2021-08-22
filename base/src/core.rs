use crate::error::Error;
use crate::node::Node;
use std::collections::HashMap;
use actix::{Addr, Arbiter, Actor, ArbiterHandle};
use crate::signal::{Heartbeat, RegisterServiceInNodeSignal};
use std::time::Duration;
use log::{info, trace, error, debug};
use crate::service::{ServiceCore};
use crate::transport::Transport;
use crate::message::Parcel;
use crate::config::ServiceConfig;
use crate::plugin::PluginManager;


type ServiceTypeName = String;

pub struct CoreBuilder<F>
    where
        F: Fn() -> Node,
{
    factory: F,

    plugins: Vec<String>,
    plugin_manager: PluginManager,
}

impl<F> CoreBuilder<F>
    where
        F: Fn() -> Node
{
    pub fn new(factory: F) -> CoreBuilder<F> {
        let builder = CoreBuilder {
            factory,
            plugin_manager: PluginManager::new(),
            plugins: vec![],
        };

        builder
    }

    pub fn plugins(&mut self, plugins: Vec<String>) -> &mut Self {
        self.plugins = plugins;
        self
    }

    pub fn build(&mut self) -> Core {
        let node = (self.factory)();

        let arbiter = Arbiter::new().handle();
        let node = Node::start_in_arbiter(&arbiter, |_ctx| {
            node
        });

        let mut core = Core {
            arbiter,
            node,
            service_factories: Default::default(),
        };

        for plugin in &self.plugins {
            info!("Loading plugin in build");
            info!("{:?}",std::fs::File::open(plugin.clone()));
            unsafe {
                match self.plugin_manager.load_plugin(plugin, &mut core) {
                    Ok(_result) => {
                        info!("Plugin loaded!");
                    }
                    Err(e) => {
                        error!("Error while loading plugin\n{:?}", e);
                    }
                }
            }
        }

        trace!("{:?}", core.service_factories.keys());

        let factory = core.service_factories.get("TelnetService").unwrap();
        trace!("aaa");
        let a = factory(ServiceConfig {
            name: "".to_string(),
            operation_config: Default::default(),
            parameters: Default::default(),
        }, core.node.clone());

        debug!("{:?}", a);

        core
    }
}

pub struct Core {
    arbiter: ArbiterHandle,
    node: Addr<Node>,
    service_factories: HashMap<ServiceTypeName, Box<extern "C" fn(ServiceConfig, Addr<Node>) -> ServiceCore>>,
}

impl Core {
    pub fn new(name: String) -> Self {
        info!("AnyMessage core started!");
        let arbiter = Arbiter::new().handle();
        let node = Node::start_in_arbiter(&arbiter, |_ctx| {
            Node::new(name)
        });

        Core {
            arbiter,
            node,
            service_factories: Default::default(),
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

    }

    pub fn node(&self) -> Addr<Node> {
        self.node.clone()
    }

    pub async fn service<F: Fn(&mut ServiceCore, Addr<Node>, &Core)>(&self, service_name: String, config: F) -> Addr<ServiceCore> {
        let mut service_core = ServiceCore::new(service_name.clone(), self.node.clone());
        config(&mut service_core, self.node.clone(), self);

        let arbiter = Arbiter::new().handle();
        let consumed_messages = service_core.get_consuming_message_types();
        let operations = service_core.get_operations().clone();

        let service_addr = ServiceCore::start_in_arbiter(&arbiter, |_ctx| {
            service_core
        });

        match self.node.send(RegisterServiceInNodeSignal {
            transport: Transport::new(service_addr.clone().recipient::<Parcel>()),
            name: service_name,
            operations: operations.clone(),
            consume_messages: consumed_messages,
        }).await {
            Err(e) => {
                error!("Error {:?}", e);
            },
            _ => {}
        }

        service_addr
    }

    pub fn service_config(&mut self, service_type_name: ServiceTypeName, service_factory: Box<extern "C" fn(ServiceConfig, Addr<Node>) -> ServiceCore>) {
        self.service_factories.insert(service_type_name, service_factory);
    }
}