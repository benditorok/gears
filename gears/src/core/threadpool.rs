use log::info;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{mpsc, Arc, Mutex},
    thread,
};

/// A job to be executed by the thread pool.
type Job = Box<dyn FnOnce(Arc<AtomicBool>) + Send + 'static>;

/// A worker for the thread pool.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new worker.
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        stop_flag: Arc<AtomicBool>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    info!("Worker {id} got a job; executing.");
                    job(Arc::clone(&stop_flag));
                }
                Err(_) => {
                    info!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

/// A thread pool for executing jobs in parallel.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    stop_flag: Arc<AtomicBool>,
}

impl ThreadPool {
    /// Create a new ThreadPool with the specified number of workers.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let stop_flag = Arc::new(AtomicBool::new(false));

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&stop_flag),
            ));
        }

        Self {
            workers,
            sender: Some(sender),
            stop_flag,
        }
    }

    /// Execute a job on the thread pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(Arc<AtomicBool>) + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .expect("Failed to send job to thread pool");
    }

    /// Stop all workers.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Resume all workers.
    pub fn resume(&self) {
        self.stop_flag.store(false, Ordering::SeqCst);
    }
}

impl Drop for ThreadPool {
    /// Stop all workers and join them.
    fn drop(&mut self) {
        // Request all workers to stop
        self.stop();

        // Drop the sender to close the channel
        self.sender.take();

        // Join all the worker threads
        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
