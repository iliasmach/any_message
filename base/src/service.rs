use crate::route::{Route, RouteSheet};
use actix::{Recipient, Handler};
use crate::signal::RegisterServiceInNodeSignal;
use crate::error::Error;
use crate::message::{BaseMessage, Parcel};

pub trait Service : Handler<Parcel> {
    fn get_route(&self) -> &Route;
    fn start_service(&mut self, name: Route, node: Recipient<RegisterServiceInNodeSignal>);
    fn can_handle_operation(&self, route_sheet: &RouteSheet, operation: Option<String>) -> Result<bool, Error>;
    fn handle_message(&mut self, message: BaseMessage);
}