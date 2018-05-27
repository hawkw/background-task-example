//! An example that demonstrates running the background work on a different
//! executor from the handle futures:
//!
//! Run it with:
//!
//! ```notrust
//! $ cargo run --example multi-executor
//! ```

extern crate background_task_example;
extern crate tokio;
extern crate futures;
#[macro_use] extern crate log;
extern crate ansi_term;
extern crate env_logger;

mod logging;

use std::thread;

fn main() {
    // Configure logging so we can see what's going on under the hood.
    logging::formatted_builder()
        .unwrap()
        .filter_level(log::LevelFilter::Info)
        .filter_module("background_task_example", log::LevelFilter::Trace)
        .init();

    let (mut handle, bg) = background_task_example::SumBackground::new();
    thread::Builder::new()
        .name("background".into())
        .spawn(move || {
            let mut rt = tokio::runtime::current_thread::Runtime::new()
                .unwrap();
            rt.block_on(bg).expect("run background task");
            info!("Background task finished.");
        })
        .expect("background thread");

    let mut rt = tokio::runtime::current_thread::Runtime::new()
        .unwrap();

    for i in 1..5 {
        info!("Adding 2 to {:?} in the background...", i);
        let sum_future = handle.sum(i, 2);
        let sum = rt.block_on(sum_future).expect("summing should not fail!");
        info!("{:?} + 2 = {:?}", i, sum);
    }
}
