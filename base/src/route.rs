use semver::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    route: String,
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


    pub fn set_node_name(&mut self, node_name: String) -> &mut Self { self.node_name = node_name; self }
    pub fn set_service_name(&mut self, service_name: String) -> &mut Self { self.service_name = service_name; self }
    pub fn set_operation_name(&mut self, operation_name: String) -> &mut Self { self.operation_name = operation_name; self }
    pub fn set_inner_id(&mut self, inner_id: String) -> &mut Self { self.inner_id = inner_id; self }

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
pub enum RouteStrategy {
    OneOf,
    All,
}

#[derive(Debug, Clone)]
pub struct RouteSheet {
    was_in: Vec<Route>,
    was_in_targets: Vec<Route>,
    target_routes: Vec<Route>,
    current_target: Option<Route>,
    next_route: Option<Route>,
    from: Route,
    route_strategy: RouteStrategy,
}

impl RouteSheet {
    pub fn new(target: Route, from: Route) -> Self {
        Self {
            was_in: vec![],
            was_in_targets: vec![],
            target_routes: vec![target],
            current_target: None,
            next_route: None,
            from,
            route_strategy: RouteStrategy::OneOf,
        }
    }

    pub fn target(&self) -> &Route {
        &self.target_routes.first().unwrap()
    }
    pub fn from(&self) -> &Route {
        &self.from
    }
    pub fn revert(&self) -> RouteSheet {
        Self {
            was_in: vec![],
            was_in_targets: vec![],
            current_target: None,
            target_routes: vec![self.from.clone()],
            from: self.current_target.clone().unwrap().clone(),
            next_route: None,
            route_strategy: RouteStrategy::OneOf,
        }
    }
}