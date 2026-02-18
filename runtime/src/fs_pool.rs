use std::sync::{mpsc, OnceLock, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

static SENDER: OnceLock<mpsc::Sender<Job>> = OnceLock::new();

pub fn init_worker_pool() {
    let (tx, rx) = mpsc::channel::<Job>();

    // shared receiver across workers
    let rx = Arc::new(Mutex::new(rx));

    // number of IO workers
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .max(2)
        .min(8);

    for _ in 0..workers {
        let rx = rx.clone();

        thread::spawn(move || {
            loop {
                let job = {
                    let lock = rx.lock().unwrap();
                    lock.recv()
                };

                match job {
                    Ok(job) => job(),
                    Err(_) => break,
                }
            }
        });
    }

    let _ = SENDER.set(tx);
}

pub fn spawn_io<F: FnOnce() + Send + 'static>(f: F) {
    if let Some(tx) = SENDER.get() {
        let _ = tx.send(Box::new(f));
    } else {
        // fallback, should never happen after runtime init
        std::thread::spawn(f);
    }
}
