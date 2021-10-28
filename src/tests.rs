#[cfg(test)]
mod message_tests {
    use crate::route::{RouteSheet, Route};
    use crate::message::Request;

    #[test]
    fn test_message() {
        // let body = "Hello!".to_string();
        // let route_sheet = RouteSheet::new(
        //     Route::new(),
        //     Route::new(),
        // );
        // let request = Request::new(body.clone().into_bytes(), route_sheet);
        // let guid = request.guid();
        //
        //
        // assert!(guid.len() > 0);
        // assert_eq!(guid.len(), 22);
        // assert_eq!(String::from_utf8(request.body().clone()).unwrap(), body);
    }
}

#[cfg(test)]
mod base_core_tests {
    use crate::core::{Core, CoreBuilder};
    use lazy_static::lazy_static;
    use crate::node::Node;
    use std::sync::{Mutex, Arc};
    use actix::{Actor, System, Arbiter};
    use std::time::Duration;
    use crate::message::{Parcel, BaseMessage};
    use crate::route::{RouteSheet, Route};
    use crate::signal::RegisterServiceInNodeSignal;
    use crate::transport::Transport;

    #[test]
    fn base_core_start() {
        dotenv::dotenv();
        env_logger::init();
        let j = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(2));
            std::process::exit(0);
        });
        System::new().block_on(async {
            CoreBuilder::new(|| {
                Node::new("default".to_string())
            }).build().await.run().await;
        });

        j.join();
    }
}