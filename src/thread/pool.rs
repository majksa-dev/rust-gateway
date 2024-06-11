use super::{job::Job, worker::Worker, ChannelClosed, ZeroThreads};
use essentials::{debug, error};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            debug!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(_) => debug!("Worker {} shut down", worker.id),
                    Err(e) => error!("Worker {} panicked: {:?}", worker.id, e),
                }
            }
        }
    }
}

macro_rules! throw_if {
    ($expr:expr, $error:expr) => {
        if $expr {
            Err($error)?
        }
    };
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, ZeroThreads> {
        throw_if!(size == 0, ZeroThreads);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let status = Arc::new(Mutex::new(vec![false; size]));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&status)));
        }
        for _ in 0..size {
            while !status.lock().unwrap().iter().all(|&x| x) {
                thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    pub fn execute<F>(&self, f: F) -> Result<(), ChannelClosed>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .and_then(|sender| sender.send(job).ok())
            .ok_or(ChannelClosed)
    }
}
