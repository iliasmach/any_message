use std::time::Duration;
use actix_rt::time::Instant;
use actix::Message;

pub trait BaseMessage: Message {
    fn is_expired(&self) -> bool {
        false
    }

    fn was_in_route(&mut self, route: String);
}

/// Class for base message, that don`t have answer
#[derive(Debug, Clone)]
pub struct BaseSignal {
    data: Vec<u8>,
    ttl: Duration,
    created: Instant,
    was_in_routes: String,
}

impl BaseSignal {
    pub fn new(data: Vec<u8>, ttl: Duration) -> Self {
        Self {
            data,
            ttl,
            created: Instant::now(),
            was_in_routes: "".to_string(),
        }
    }
}

impl BaseMessage for BaseSignal {
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.created) > self.ttl
    }

    fn was_in_route(&mut self, route: String) {
        if self.was_in_routes.is_empty() {
            self.was_in_routes = route;
        } else {
            self.was_in_routes = format!("{}.{}", self.was_in_routes, route);
        }
    }
}

impl Message for BaseSignal {
    type Result = ();
}