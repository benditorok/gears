use log::info;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce(Arc<AtomicBool>) + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        stop_flag: Arc<AtomicBool>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(Message::NewJob(job)) => {
                    info!("Worker {id} got a job; executing.");

                    job(Arc::clone(&stop_flag));
                }
                Ok(Message::Terminate) => {
                    info!("Worker {id} was told to terminate.");
                    break;
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

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
    stop_flag: Arc<AtomicBool>,
}

impl ThreadPool {
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

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(Arc<AtomicBool>) + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .as_ref()
            .unwrap()
            .send(Message::NewJob(job))
            .expect("Failed to send job to thread pool");
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.stop_flag.store(false, Ordering::SeqCst);
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to close the channel
        if let Some(sender) = self.sender.take() {
            // Send the terminate message to each worker
            for _ in &self.workers {
                sender.send(Message::Terminate).unwrap();
            }
        }

        // Join all the worker threads
        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
