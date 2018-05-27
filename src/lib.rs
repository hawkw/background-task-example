extern crate futures;
#[macro_use]
extern crate log;

use futures::{
    sync::{mpsc, oneshot},
    Future, Async, Poll, Stream,
};

// A sum request consists of:
#[derive(Debug)]
struct SumRequest {
    // Two numbers to add together...
    a: usize,
    b: usize,
    // ...and a `oneshot::Sender` for returning the sum.
    result_tx: oneshot::Sender<usize>,
}


/// A background task that handles `SumRequest`s.
#[derive(Debug)]
pub struct SumBackground {
    rx: mpsc::UnboundedReceiver<SumRequest>,
}

/// A handle for communicating with a `SumTask`.
#[derive(Clone, Debug)]
pub struct SumHandle {
    tx: mpsc::UnboundedSender<SumRequest>,
}

// ===== impl SumHandle =====

impl SumHandle {
    /// Add two numbers `a` and `b` in the background and return the sum.
    pub fn sum(&mut self, a: usize, b: usize) -> impl Future<Item = usize, Error = ()> {
        // Construct a new oneshot channel on which we'll receive the result.
        let (result_tx, result_rx) = oneshot::channel();
        // Build  the request to send to the background task.
        let request = SumRequest {
            a,
            b,
            result_tx,
        };
        // Send the request.
        self.tx.unbounded_send(request)
            .expect("sending request to background task failed!");
        // Return the receive side of the oneshot channel, which is a
        //`Future<Item = usize>`.
        trace!("SumHandle::sum: request for {:?} + {:?} sent.", a, b);
        result_rx
            // The receiver should _not_ be dropped while there are
            // still `SumHandle`s.
            .map_err(|e| panic!("background task cancelled unexpectedly: {:?}", e))
            // Map over the result so we can log it.
            .map(|result| {
                trace!("SumHandle::sum: Received result: {:?}",  result);
                result
            })
    }
}

// ===== impl SumBackground =====

impl SumBackground {
    pub fn new() -> (SumHandle, Self) {
        let (tx, rx) = mpsc::unbounded();
        let background = SumBackground { rx };
        let handle = SumHandle { tx };
        (handle, background)
    }
}

impl Future for SumBackground {
    // In order to be `spawn`ed in the background, the `Item` and
    // `Error` types of the future *must* be unit.
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        // We'll use a loop to continue polling `rx` until it's  `NotReady`.
        loop {
            trace!("SumBackground::poll: Polling for new requests...");
            match self.rx.poll() {
                // When `rx` is `NotReady`, this means there are no more
                // incoming requests, so the background task should yield
                // until there are.
                Ok(Async::NotReady) => {
                    trace!("--> No new requests, yielding.");
                    return Ok(Async::NotReady);
                }
                // If the request stream has ended, then there are no more
                // handles, and the background task can complete successfully.
                Ok(Async::Ready(None)) => {
                    trace!("--> Request handles are gone; ending background task.");
                    return Ok(Async::Ready(()));
                },
                // We've gotten a request!
                Ok(Async::Ready(Some(SumRequest { a, b, result_tx }))) => {
                    trace!("--> Received request to calculate: {:?} + {:?}", a, b);
                    // Sum the two numbers (this is where a more complex
                    // background task might compose a new `Future` to return).
                    let sum = a + b;
                    // Send the result on the oneshot channel included with the
                    // request.
                    if let Err(_) = result_tx.send(sum) {
                        // `oneshot::send` fails if the receiver was dropped,
                        // which is fine, so rather than panicking, just log
                        // that the request was canceled.
                        // (a more complex background task might clean up some
                        // state here...)
                        debug!("SumBackground::poll: Request canceled before it was completed");
                    }
                    trace!("--> Result sent.");
                    // Don't return anything, just continue looping until
                    // there are no more incoming requests.
                }
                Err(e) => {
                    // The unbounded receiver should not error.
                    warn!("SumBackground::poll: Request channel error: {:?}", e);
                    return Err(());
                }
            }
        }
    }
}
