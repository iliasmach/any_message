use semver::Version;
use crate::message::BaseMessage;
use crate::error::Error;

#[derive(Clone, Debug, Hash, Eq)]
pub struct Operation {
    name: String,
    version: Version,
    description: String,
}

impl Operation {
    pub fn new(name:String, version: Version, description: String) -> Self {
        Self {
            name,
            version,
            description,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

pub enum OperationError {
    OperationNotFound,
    OperationNotFoundInNode,
    OperationNotFoundInService,
}

impl PartialEq for Operation {
    fn eq(&self, other: &Self) -> bool {
        other.name == self.name && other.version == self.version
    }

    fn ne(&self, other: &Self) -> bool {
        other.name != self.name || other.version != self.version
    }
}

pub struct OperationHandler<F> {
    pub handler: F,
}