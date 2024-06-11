use super::job::Job;
use essentials::{debug, error, info};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Worker {
    pub id: usize,
    pub thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        status: Arc<Mutex<Vec<bool>>>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            info!("Worker {id} connected.");
            *status.lock().unwrap().get_mut(id).unwrap() = true;
            loop {
                debug!("Worker {id} waiting for job.");
                let message = match receiver.lock() {
                    Ok(receiver) => receiver.recv(),
                    Err(err) => {
                        error!("Worker {id} error: {err}");
                        continue;
                    }
                };
                match message {
                    Ok(job) => {
                        debug!("Worker {id} received a job; executing.");
                        job();
                    }
                    Err(_) => {
                        info!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                };
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
