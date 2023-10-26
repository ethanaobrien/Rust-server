use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use openssl::ssl::{SslMethod, SslAcceptor};
use openssl::rsa::Rsa;
use openssl::x509::X509;
use openssl::pkey::PKey;

fn to_acceptor(cert_str: &str, key_str: &str) -> SslAcceptor {
    
    let cert_str = cert_str
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    let key_str = key_str
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    let cert = X509::from_pem(cert_str.as_bytes()).unwrap();
    let key = Rsa::private_key_from_pem(key_str.as_bytes()).unwrap();
    
    let pkey = PKey::from_rsa(key).unwrap();
    
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key(&pkey).unwrap();
    builder.set_certificate(&cert).unwrap();
    
    builder.build()
}

pub struct TlsThreadPool {
    workers: Vec<TlsWorker>,
    sender: Option<mpsc::Sender<TlsJob>>,
    enabled: bool
}

type TlsJob = Box<dyn FnOnce(&SslAcceptor) + Send + 'static>;

impl TlsThreadPool {
    pub fn new(enabled: bool, size: usize, https_cert: &str, https_key: &str) -> TlsThreadPool {
        if !enabled { return TlsThreadPool {
            workers: Vec::new(),
            sender: None,
            enabled: false
        }; }
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(TlsWorker::new(id, Arc::clone(&receiver), to_acceptor(https_cert, https_key)));
        }

        TlsThreadPool {
            workers,
            sender: Some(sender),
            enabled: true
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&SslAcceptor) + Send + 'static,
    {
        if !self.enabled { return }; 
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for TlsThreadPool {
    fn drop(&mut self) {
        if !self.enabled { return };
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct TlsWorker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl TlsWorker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<TlsJob>>>, acceptor: SslAcceptor) -> TlsWorker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    job(&acceptor);
                }
                Err(_) => {
                    break;
                }
            }
        });

        TlsWorker {
            id,
            thread: Some(thread),
        }
    }
}



pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    enabled: bool
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(enabled: bool, size: usize) -> ThreadPool {
        if !enabled { return ThreadPool {
            workers: Vec::new(),
            sender: None,
            enabled: false
        }; }
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
            enabled: true
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if !self.enabled { return }; 
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if !self.enabled { return };
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    job();
                }
                Err(_) => {
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
