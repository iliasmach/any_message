use actix::Message;
use crate::route::{RouteSheet, Target};
use std::time::Duration;
use crate::operation::Operation;
use log::trace;


#[derive(Debug, Clone)]
pub struct BaseMessage {
    data: Vec<u8>,
    operation: Option<Operation>,
}

impl BaseMessage {
    pub fn new(data: Vec<u8>, operation: Option<Operation>) -> Self {
        Self { data, operation }
    }

    pub fn operation(&self) -> Option<Operation> {
        self.operation.clone()
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

// pub struct Envelope {
//     message: BaseMessage,
//     route_sheet: RouteSheet
// }

#[derive(Debug)]
pub struct Parcel {
    route_sheet: RouteSheet,
    messages: Vec<BaseMessage>,
    ttl: Option<Duration>,
}

impl Clone for Parcel {
    fn clone(&self) -> Self {
        trace!("Cloning parcel");
        Self {
            route_sheet: self.route_sheet.clone(),
            messages: self.messages.clone(),
            ttl: self.ttl.clone()
        }
    }

    fn clone_from(&mut self, source: &Self) {
        trace!("Cloning parcel");

        self.ttl = source.ttl.clone();
        self.route_sheet = source.route_sheet.clone();
        self.messages = source.messages.clone();
    }
}

impl Parcel {
    pub fn new(messages: Vec<BaseMessage>, route_sheet: RouteSheet) -> Self {
        Self { route_sheet, messages, ttl: None }
    }

    pub fn target(&self) -> &Target {
        self.route_sheet.target()
    }

    pub fn route_sheet(&self) -> &RouteSheet {
        &self.route_sheet
    }

    pub fn unpack(&self) -> &Vec<BaseMessage> {
        &self.messages
    }
}

// impl Message for Envelope {
//     type Result = ();
// }

impl Message for Parcel {
    type Result = ();
}

pub struct Request {
    body: Vec<u8>,
    route_sheet: RouteSheet,
    guid: String,
}

impl Message for Request {
    type Result = ();
}

impl Request {
    pub fn new(body: Vec<u8>, route_sheet: RouteSheet) -> Self {
        Self {
            body,
            route_sheet,
            guid: nano_id::base64(22)
        }
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn guid(&self) -> &String {
        &self.guid
    }

    pub fn route_sheet(&self) -> &RouteSheet {
        &self.route_sheet
    }
}
