use uuid::Version;
use std::path::Path;
use std::collections::HashMap;

pub struct ConfigBuilder {
    config_string: String,
}

impl ConfigBuilder {
    pub fn from_string(string: String) -> Self {
        let builder = Self {
            config_string: string
        };

        builder
    }

    pub fn from_file(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(string) => return Self::from_string(string),
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    pub fn build(&mut self) -> CoreConfig {
        CoreConfig{ name: "".to_string(), node_config: NodeConfig { name: "".to_string(), service_config: ServiceConfig {
            name: "".to_string(),
            operation_config: OperationConfig {
                name: "".to_string(),
                version: Version::Nil,
                description: "".to_string()
            },
            parameters: Default::default()
        } } }
    }
}

pub struct CoreConfig {
    name: String,
    node_config: NodeConfig,
}

impl CoreConfig {
    pub fn new(name: String, config_string: String) {

    }
}

pub struct NodeConfig {
    name: String,
    service_config: ServiceConfig,
}

pub struct ServiceConfig {
    name: String,
    operation_config: OperationConfig,
    parameters: HashMap<String, String>,
}

pub struct OperationConfig {
    name: String,
    version: Version,
    description: String
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        
    }
}