
pub mod exchange;
pub mod message;
pub mod receiver;
pub mod source;
pub mod acrix_actor;
pub mod rabbit_mq;
pub mod store;

pub struct HandlerContext {}

#[cfg(test)]
mod tests {
    use actix::{Actor, Context};
    use std::time::Duration;
    use crate::message::BaseSignal;
    use crate::acrix_actor::ActorMessageSource;
    use crate::rabbit_mq::RabbitMessageReceiver;
    use crate::exchange::{BaseMessageExchange, FanoutExchange, RunExchange};
    use crate::HandlerContext;
    use crate::source::{MessageSource, GetMessagesFromSource};

    fn handle_context(message: BaseSignal, _ctx: &mut HandlerContext) -> Result<BaseSignal, Box<dyn std::error::Error>> {
        println!("Handling message {:?}", message);
        Ok(message)
    }

    #[actix_rt::test]
    async fn base_exchange() {
        let source = ActorMessageSource::start(
            ActorMessageSource::new(
                "actor".to_string()
            )
        );
        let receiver = RabbitMessageReceiver::start(RabbitMessageReceiver {
            last_message: None
        });
        let mut exchange = BaseMessageExchange::new("exchange".to_string(), source.clone(), HandlerContext {}, Box::new(handle_context), receiver);
        source.send(BaseSignal::new(String::from("Привет!").into_bytes(), Duration::from_secs(5))).await.unwrap();
        exchange.run_once().await;
    }

    #[actix_rt::test]
    async fn fanout_exchange() {
        let source = ActorMessageSource::start(
            ActorMessageSource::new(
                "actor".to_string()
            )
        );
        let receiver = RabbitMessageReceiver::start(RabbitMessageReceiver {
            last_message: None
        }).recipient();

        let receiver2 = RabbitMessageReceiver::start(RabbitMessageReceiver {
            last_message: None
        }).recipient();
        // receiver.do_send(BaseSignal::new(String::from("Привет!").into_bytes(), Duration::from_secs(5)));
        let exchange = FanoutExchange::new(
            "fanout".to_string(),
                                               source.clone().recipient::<GetMessagesFromSource<BaseSignal>>(),
        vec![receiver.clone(), receiver2.clone()]);
         source.send(BaseSignal::new(String::from("Привет!").into_bytes(), Duration::from_secs(5))).await.unwrap();

        Context::new().run(exchange).send(RunExchange {}).await.unwrap();

    }

}
