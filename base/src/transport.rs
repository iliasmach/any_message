use actix::Recipient;
use crate::message::Parcel;

#[derive(Clone)]
pub struct Transport {
    target: Recipient<Parcel>,
    is_open: bool,
}

impl Transport {
    pub fn new(target: Recipient<Parcel>) -> Transport {
        Transport { target, is_open: true }
    }

    pub async fn send_parcel(&self, parcel: &Parcel) {
        match self.target.send(parcel.clone()).await {
            Ok(result) => {},
            Err(e) => {

            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}