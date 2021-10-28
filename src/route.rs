use semver::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    pub route: String,
    node_name: String,
    service_name: String,
    operation_name: String,
    operation_version: Option<Version>,
    inner_id: String,
}

impl Route {
    pub fn new() -> Self {
        Self {
            route: "".to_string(),
            node_name: "".to_string(),
            service_name: "".to_string(),
            operation_name: "".to_string(),
            operation_version: None,
            inner_id: "".to_string(),
        }
    }


    pub fn node_name(&self) -> &String {
        &self.node_name
    }
    pub fn service_name(&self) -> &String {
        &self.service_name
    }
    pub fn operation_name(&self) -> &String {
        &self.operation_name
    }
    pub fn inner_id(&self) -> &String {
        &self.inner_id
    }


    pub fn set_node_name(&mut self, node_name: String) -> &mut Self {
        self.node_name = node_name;
        self
    }
    pub fn set_service_name(&mut self, service_name: String) -> &mut Self {
        self.service_name = service_name;
        self
    }
    pub fn set_operation_name(&mut self, operation_name: String) -> &mut Self {
        self.operation_name = operation_name;
        self
    }
    pub fn set_inner_id(&mut self, inner_id: String) -> &mut Self {
        self.inner_id = inner_id;
        self
    }

    pub fn as_string(&self) -> String {
        let mut route = String::new();

        if !self.node_name.is_empty() {
            route.push_str("@");
            route.push_str(self.node_name.as_str());
        }

        if !self.service_name.is_empty() {
            route.push_str("::");
            route.push_str(self.service_name.as_str());
        }

        if !self.operation_name.is_empty() {
            route.push_str("/");
            route.push_str(self.operation_name.as_str());
        }

        if !self.inner_id.is_empty() {
            route.push_str(":");
            route.push_str(self.inner_id.as_str());
        }

        route
    }
}

#[derive(Debug, Clone)]
pub struct RouteSheet {
    target: Target,
    from: Route,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Target {
    Route(Route),
    Consumer(String),
}

impl Target {
    pub fn as_string(&self) -> String {
        match self {
            Target::Route(route) => {
                route.as_string()
            },
            Target::Consumer(name) => {
                format!("Consumer({})", name.to_string())
            }
        }
    }
}

impl RouteSheet {
    pub fn new(target: Target, from: Route) -> Self {
        Self {
            target,
            from,
        }
    }

    pub fn target(&self) -> &Target {
        &self.target
    }
    pub fn from(&self) -> &Route {
        &self.from
    }
}