use actix::Recipient;
use crate::message::Parcel;
use log::{trace, error};

#[derive(Clone, Debug)]
pub struct Transport {
    target: Recipient<Parcel>,
    is_open: bool,
}

impl Transport {
    pub fn new(target: Recipient<Parcel>) -> Transport {
        Transport { target, is_open: true }
    }

    pub fn send_parcel(&self, parcel: Parcel) {
        trace!("Sending parcel");
        self.target.do_send(parcel);
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}