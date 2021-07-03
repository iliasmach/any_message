use actix::Message;
use crate::route::{RouteSheet, Route};
use std::time::Duration;



#[derive(Debug, Clone)]
pub struct BaseMessage {
    data: Vec<u8>,
    operation: Option<String>,
}

impl BaseMessage {
    pub fn new(data: Vec<u8>, operation: Option<String>) -> Self {
        Self { data, operation }
    }

    pub fn operation(&self) -> Option<String> {
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

#[derive(Debug, Clone)]
pub struct Parcel {
    route_sheet: RouteSheet,
    messages: Vec<BaseMessage>,
    ttl: Option<Duration>,
}

impl Parcel {
    pub fn new(messages: Vec<BaseMessage>, route_sheet: RouteSheet) -> Self {
        Self { route_sheet, messages, ttl: None }
    }

    pub fn target(&self) -> &Route {
        self.route_sheet.target()
    }

    pub fn route_sheet(&self) -> &RouteSheet {
        &self.route_sheet
    }

    pub fn unpack(&self) -> Vec<BaseMessage> {
        self.messages.clone()
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

#[cfg(test)]
mod test_messages {
    use crate::message::Request;
    use crate::route::{RouteSheet, Route};

    #[test]
    fn test_message() {
        let body = "Hello!".to_string();
        let route_sheet = RouteSheet::new(
            Route::new("target".to_string()),
            Route::new("this".to_string())
        );
        let request = Request::new(body.clone().into_bytes(), route_sheet);
        let guid = request.guid();


        assert!(guid.len() > 0);
        assert_eq!(guid.len(), 22);
        assert_eq!(String::from_utf8(request.body).unwrap(), body);
    }



}