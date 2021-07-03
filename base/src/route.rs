#[derive(Debug, Clone)]
pub struct Route {
    name: String,
    node_name: String,
    service_name: String,
    operation_name: String,
    inner_id: String,
}

impl Route {
    pub fn new(name: String) -> Self {
        Self {
            name,
            node_name: "".to_string(),
            service_name: "".to_string(),
            operation_name: "".to_string(),
            inner_id: "".to_string(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct RouteSheet {
    was_in: Vec<Route>,
    target_route: Route,
    from: Route,
}

impl RouteSheet {
    pub fn new(target: Route, from: Route) -> Self {
        Self { was_in: vec![], target_route: target, from }
    }

    pub fn target(&self) -> &Route {
        &self.target_route
    }
    pub fn from(&self) -> &Route {
        &self.from
    }
    pub fn revert(&self) -> RouteSheet {
        Self {
            was_in: vec![],
            target_route: self.from.clone(),
            from: self.target_route.clone(),
        }
    }
}