//! A simple example that runs the background task and returned futures
//! on the same executor.
//!
//! Run it with:
//!
//! ```notrust
//! $ cargo run --example simple
//! ```

extern crate background_task_example;
extern crate tokio;
extern crate futures;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

fn main() {
    // Configure logging so we can see what's going on under the hood.
    pretty_env_logger::formatted_builder()
        .unwrap()
        .filter_level(log::LevelFilter::Info)
        .filter_module("background_task_example", log::LevelFilter::Trace)
        .init();

    let mut rt = tokio::runtime::current_thread::Runtime::new()
        .unwrap();

    let (mut handle, bg) = background_task_example::SumBackground::new();
    rt.spawn(bg);

    for i in 1..5 {
        info!("Adding 2 to {:?} in the background...", i);
        let sum_future = handle.sum(i, 2);
        let sum = rt.block_on(sum_future).expect("summing should not fail!");
        info!("{:?} + 2 = {:?}", i, sum);
    }
}
