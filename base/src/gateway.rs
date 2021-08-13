use actix::prelude::*;
use crate::message::Parcel;

pub trait MessageSource {
    fn message_types() -> Vec<String>;

}

pub trait GateWay : Actor + Handler<Parcel> {

}