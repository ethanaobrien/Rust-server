use std::{
    sync::{
        mpsc,
        mpsc::Receiver,
        Arc,
        Mutex
    },
    thread,
};
use openssl::{
    ssl::{SslMethod, SslAcceptor},
    rsa::Rsa,
    x509::X509,
    pkey::PKey
};

fn to_acceptor(cert_str: &str, key_str: &str) -> Result<SslAcceptor, openssl::error::ErrorStack> {
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

    let cert = X509::from_pem(cert_str.as_bytes())?;
    let key = Rsa::private_key_from_pem(key_str.as_bytes())?;
    
    let pkey = PKey::from_rsa(key)?;
    
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key(&pkey)?;
    builder.set_certificate(&cert)?;
    
    Ok(builder.build())
}

pub struct TlsThreadPool {
    workers: Vec<TlsWorker>,
    sender: Option<mpsc::Sender<TlsJob>>,
    enabled: bool,
    threads: usize,
    active_threads: Arc<Mutex<usize>>,
    receiver: Arc<Mutex<Receiver<Box<dyn FnOnce(&SslAcceptor) + Send>>>>,
    https_cert: String,
    https_key: String,
    max_threads: usize
}

macro_rules! new_job {
    ($count:expr, $start:expr) => {
        let mut retry_count = 0;
        loop {
            if retry_count >= 10 {
                return;
            }
            let Ok(mut locked) = $count.lock() else {
                retry_count += 1;
                continue;
            };
            if $start {
                *locked += 1;
            } else if *locked > 0 {
                *locked -= 1;
            }
            break;
        }
    };
}

type TlsJob = Box<dyn FnOnce(&SslAcceptor) + Send + 'static>;

impl TlsThreadPool {
    pub fn new(enabled: bool, max_threads: usize, https_cert: &str, https_key: &str) -> TlsThreadPool {
        let (sender, receiver) = mpsc::channel();
        if !enabled { return TlsThreadPool {
            workers: Vec::new(),
            sender: None,
            enabled: false,
            threads: 0,
            active_threads: Arc::new(Mutex::new(0)),
            receiver: Arc::new(Mutex::new(receiver)),
            https_key: String::new(),
            https_cert: String::new(),
            max_threads: 0
        }; }

        let receiver = Arc::new(Mutex::new(receiver));
        let start : usize = 0;
        let active_threads = Arc::new(Mutex::new(start));

        let mut workers = Vec::new();

        let mut retry_count = 0;
        loop {
            if retry_count > 10 { break; };
            let Ok(acceptor) = to_acceptor(https_cert, https_key) else {
                retry_count += 1;
                continue;
            };
            workers.push(TlsWorker::new(0, Arc::clone(&receiver), acceptor, active_threads.clone()));
            break;
        }

        TlsThreadPool {
            workers,
            sender: Some(sender),
            enabled: true,
            threads: 1,
            active_threads: active_threads,
            receiver: receiver,
            https_cert: https_cert.to_string(),
            https_key: https_key.to_string(),
            max_threads: max_threads
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce(&SslAcceptor) + Send + 'static,
    {
        if !self.enabled { return }; 
        let job = Box::new(f);

        let mut retry = 0;
        loop {
            if retry > 10 {
                println!("Critical error occured! Cannot lock active_threads!");
                return;
            }
            let Ok(active_threads) = self.active_threads.lock() else {
                retry += 1;
                continue;
            };

            if *active_threads >= self.threads && self.threads <= self.max_threads {
                
                let mut retry_count = 0;
                loop {
                    if retry_count > 10 { break; };
                    let Ok(acceptor) = to_acceptor(&self.https_cert, &self.https_key) else {
                        retry_count += 1;
                        continue;
                    };
                    self.workers.push(TlsWorker::new(self.threads, Arc::clone(&self.receiver), acceptor, self.active_threads.clone()));
                    self.threads += 1;
                    break;
                }
                //println!("Created a new thread: {}", self.threads);
            }

            let mut retry_count = 0;
            loop {
                if retry_count > 10 { break; };
                let Some(sender) = self.sender.as_ref() else {
                    retry_count += 1;
                    continue;
                };
                let Ok(_) = sender.send(job) else {
                    println!("Critical Error! Failed to send job to worker!");
                    return;
                };
                break;
            }
            break;
        }
    }
}

#[allow(dead_code)]
struct TlsWorker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl TlsWorker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<TlsJob>>>, acceptor: SslAcceptor, count: Arc<Mutex<usize>>) -> TlsWorker {
        let thread = thread::spawn(move || {
            let mut last_err = false;
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        new_job!(count, true);
                        job(&acceptor);
                        new_job!(count, false);
                    }
                    Err(_) => {
                        if last_err { break; }
                        last_err = true;
                    }
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
    enabled: bool,
    threads: usize,
    active_threads: Arc<Mutex<usize>>,
    receiver: Arc<Mutex<Receiver<Box<dyn FnOnce() + Send>>>>,
    max_threads: usize
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(enabled: bool, max_threads: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        if !enabled { return ThreadPool {
            workers: Vec::new(),
            sender: None,
            enabled: false,
            threads: 0,
            active_threads: Arc::new(Mutex::new(0)),
            receiver: Arc::new(Mutex::new(receiver)),
            max_threads: 0
        }; }

        let receiver = Arc::new(Mutex::new(receiver));
        let start : usize = 0;
        let active_threads = Arc::new(Mutex::new(start));

        let mut workers = Vec::new();

        workers.push(Worker::new(0, Arc::clone(&receiver), active_threads.clone()));

        ThreadPool {
            workers,
            sender: Some(sender),
            enabled: true,
            threads: 1,
            active_threads: active_threads,
            receiver: receiver,
            max_threads: max_threads
        }
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if !self.enabled { return }; 
        let job = Box::new(f);

        let mut retry = 0;
        loop {
            if retry > 10 {
                println!("Critical error occured! Cannot lock active_threads!");
                return;
            }
            let Ok(active_threads) = self.active_threads.lock() else {
                retry += 1;
                continue;
            };

            if *active_threads >= self.threads && self.threads <= self.max_threads {
                self.workers.push(Worker::new(self.threads, Arc::clone(&self.receiver), self.active_threads.clone()));
                //println!("Created a new thread: {}", self.threads);
                self.threads += 1;
            }

            let mut retry_count = 0;
            loop {
                if retry_count > 10 { break; };
                let Some(sender) = self.sender.as_ref() else {
                    retry_count += 1;
                    continue;
                };
                let Ok(_) = sender.send(job) else {
                    println!("Critical Error! Failed to send job to worker!");
                    return;
                };
                break;
            }
            break;
        }
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, count: Arc<Mutex<usize>>) -> Worker {
        let thread = thread::spawn(move || {
            let mut last_err = false;
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        new_job!(count, true);
                        job();
                        new_job!(count, false);
                    }
                    Err(_) => {
                        if last_err { break; }
                        last_err = true;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
