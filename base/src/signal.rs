use actix::Message;
use crate::message::Parcel;
use crate::error::Error;
use crate::route::Route;
use crate::transport::Transport;

pub struct GetMessagesSignal { pub send_to: Route }
impl Message for GetMessagesSignal { type Result = Result<Option<Parcel>, Error>; }
#[allow(dead_code)]
pub struct HasRouteSignal { route: Route }
impl Message for HasRouteSignal { type Result = Result<bool, Error>; }

pub struct RegisterServiceInNodeSignal { pub transport: Transport }
impl Message for RegisterServiceInNodeSignal { type Result = (); }