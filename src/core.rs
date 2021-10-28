use crate::error::Error;
use crate::node::Node;
use std::collections::HashMap;
use actix::{Addr, Arbiter, Actor, ArbiterHandle};
use crate::signal::{Heartbeat, RegisterServiceInNodeSignal};
use std::time::Duration;
use log::{info, trace, error, debug};
use crate::service::{Service, ServiceCore, ServiceFunctions};
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
    service_builders: HashMap<String, Box<fn(Addr<Node>) -> Box<dyn Service>>>,
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
            service_builders: HashMap::default(),
        };

        builder
    }

    pub fn plugins(&mut self, plugins: Vec<String>) -> &mut Self {
        self.plugins = plugins;
        self
    }

    pub fn service(&mut self, service_name: String, config: fn(Addr<Node>) -> Box<dyn Service>) -> &mut Self {
        self.service_builders.insert(service_name, Box::from(config));

        self
    }

    pub async fn build(&mut self) -> Core {
        let node = (self.factory)();

        let arbiter = Arbiter::new().handle();
        let node = Node::start_in_arbiter(&arbiter, |_ctx| {
            node
        });


        let mut core = Core {
            arbiter,
            node: node.clone(),
            service_factories: Default::default(),
            services: Default::default(),
        };

        for (service_name, service_builder) in &self.service_builders {
            let mut service = service_builder(node.clone());

            let mut service_core = ServiceCore::new(service_name.clone(), node.clone());
            service.config_system(&mut service_core, node.clone());

            let arbiter = Arbiter::new().handle();
            let consumed_messages = service_core.get_consuming_message_types();
            let operations = service_core.get_operations().clone();

            let service_addr = ServiceCore::start_in_arbiter(&arbiter, |_ctx| {
                service_core
            });

            match node.send(RegisterServiceInNodeSignal {
                transport: Transport::new(service_addr.clone().recipient::<Parcel>()),
                name: service_name.clone(),
                operations: operations.clone(),
                consume_messages: consumed_messages,
            }).await {
                Err(e) => {
                    error!("Error {:?}", e);
                }
                _ => {}
            }
        }

        for plugin in &self.plugins {
            trace!("Loading plugin in build");
            trace!("{:?}",std::fs::File::open(plugin.clone()));
            unsafe {
                match self.plugin_manager.load_plugin(plugin, &mut core) {
                    Ok(_result) => {
                        trace!("Plugin loaded!");
                    }
                    Err(e) => {
                        error!("Error while loading plugin\n{:?}", e);
                    }
                }
            }
        }


        core
    }
}

pub struct Core {
    arbiter: ArbiterHandle,
    node: Addr<Node>,
    service_factories: HashMap<ServiceTypeName, Box<extern "C" fn(&ServiceConfig) -> Result<ServiceFunctions, Box<dyn std::error::Error>>>>,
    services: HashMap<String, Addr<ServiceCore>>,
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
            services: Default::default(),
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



    pub fn service_config(&mut self, service_type_name: ServiceTypeName, service_factory: Box<extern "C" fn(&ServiceConfig) -> Result<ServiceFunctions, Box<dyn std::error::Error>>>) {
        self.service_factories.insert(service_type_name, service_factory);
    }
}