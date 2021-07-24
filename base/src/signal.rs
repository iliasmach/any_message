use actix::Message;
use crate::message::Parcel;
use crate::error::Error;
use crate::route::Route;
use crate::transport::Transport;
use std::time::Instant;

pub struct GetMessagesSignal { pub send_to: Route }
impl Message for GetMessagesSignal { type Result = Result<Option<Parcel>, Error>; }
#[allow(dead_code)]
pub struct HasRouteSignal { route: Route }
impl Message for HasRouteSignal { type Result = Result<bool, Error>; }

pub struct RegisterServiceInNodeSignal { pub transport: Transport, pub name: String }
impl Message for RegisterServiceInNodeSignal { type Result = (); }

pub struct Heartbeat {}
impl Message for Heartbeat { type Result = (); }

pub struct Tick {
    time: Instant,
}

impl Tick {
    pub fn new() -> Self {
        Self {
            time: Instant::now()
        }
    }

    pub fn time(&self) -> &Instant {
        &self.time
    }
}

impl Message for Tick {
    type Result = ();
}

pub struct GetRoute {}
impl Message for GetRoute { type Result = Route; }