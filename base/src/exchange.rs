use actix::{Actor, Context, Recipient, Handler};
use crate::message::Parcel;
use std::option::Iter;
use std::collections::VecDeque;
use log::{error, trace};

pub enum ExchangeType {
    RoundRobin,
    Fanout,
    Hash,
}

pub struct Exchange {
    exchange_type: ExchangeType,
    recipients: VecDeque<Recipient<Parcel>>,
    next: usize,
    length: usize
}

impl Exchange {
    pub fn new(exchange_type: ExchangeType) -> Self {
        Self { exchange_type, recipients: VecDeque::new(), next: 0, length: 0 }
    }
}

impl Actor for Exchange {
    type Context = Context<Self>;
}

impl Handler<Parcel> for Exchange {
    type Result = ();

    fn handle(&mut self, msg: Parcel, ctx: &mut Self::Context) -> Self::Result {
        match self.exchange_type {
            ExchangeType::Fanout => {
                for recipient in &self.recipients {
                    recipient.do_send(msg.clone());
                }
            }
            ExchangeType::RoundRobin => {
                if self.next >= self.length {
                    self.next = 0;
                }

                match self.recipients.get(self.next) {
                    Some(recipient) => {
                        recipient.do_send(msg.clone());
                    },
                    None => {
                        error!("Can`t send message to recipient {}", self.next);
                    }
                }
            },
            _ => {}
        }
    }
}