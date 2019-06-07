use std::thread;
use std::sync::{ mpsc, Arc, Mutex };


pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
    ///Create a new ThreadPool
    /// 
    /// Its size is the number of threads in the pool
    /// 
    /// #Panics
    /// 
    /// The 'new' function panics if the size is 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);

        let (sender, reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&reciever)));
        }
        
        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>
}

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job =  reciever.lock().unwrap().recv().unwrap();

                job.call_box();
            }
        });

        Worker {
            id,
            thread,
        }
    }
}