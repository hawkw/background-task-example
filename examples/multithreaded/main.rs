//! An example that demonstrates running the background work on one thread, and
//! cloning the handle into multiple threads which all make requests.
//!
//! Run it with:
//!
//! ```notrust
//! $ cargo run --example multithreaded
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

    // Spawn a thread to run the background task.
    let bg_thread = thread::Builder::new()
        .name("background".into())
        .spawn(move || {
            let mut rt = tokio::runtime::current_thread::Runtime::new()
                .unwrap();
            trace!("Spawning background task...");
            rt.block_on(bg).expect("run background task");
            trace!("Background task finished.");
        })
        .expect("background thread");

    let threads: Vec<_> = (1..5).map(|i| {
        trace!("Spawning thread {:?}", i);
        // Clone the handle to move into this thread's closure.
        let thread_handle = handle.clone();
        let join = thread::Builder::new()
            .name(format!("thread {:?}", i).into())
            .spawn(move || {
                let mut handle = thread_handle;
                let mut rt = tokio::runtime::current_thread::Runtime::new()
                    .unwrap();

                info!("Sending request for {:?} + {:?}", i, i + 1);
                let sum_future = handle.sum(i, i + 1);
                trace!("Waiting for result...");

                rt.block_on(sum_future).expect("summing should not fail!")
            })
            .unwrap_or_else(|e| panic!("failed to spawn thread {}: {}", i, e));
        info!("Thread {} spawned.", i);
        join
    }).collect();

    for (i, thread) in threads.into_iter().enumerate() {
        let i = i + 1;
        let result = thread.join()
            .unwrap_or_else(|e| panic!("Thread {} panicked: {:?}", i, e));
        info!("Thread {} result: {} + {} = {}", i, i, i + 1, result);
    }

    let mut rt = tokio::runtime::current_thread::Runtime::new()
        .unwrap();
    info!("Sending request for 10 + 35");
    let sum_future = handle.sum(10, 35);
    let sum = rt.block_on(sum_future).expect("summing should not fail!");
    info!("main thread result: {:?}",  sum);

    // Drop the handle so the background task finishes.
    drop(handle);
    trace!("handle dropped.");

    bg_thread.join()
        .unwrap_or_else(|e| panic!("Background thread panicked: {:?}", e));

}
