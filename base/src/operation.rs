use semver::Version;

#[derive(Clone, Debug)]
pub struct Operation {
    name: String,
    version: Version,
    description: String,
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