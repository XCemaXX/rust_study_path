use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

struct Fork;

struct Philosopher {
    name: String,
    left_fork: Arc<Mutex<Fork>>,
    right_fork: Arc<Mutex<Fork>>,
    thoughts: mpsc::Sender<String>,
}

impl Philosopher {
    fn new(name: &str, left_fork: Arc<Mutex<Fork>>, right_fork: Arc<Mutex<Fork>>, sender: mpsc::Sender<String>) -> Self {
        Philosopher {name: name.to_string(), left_fork, right_fork, thoughts: sender}
    }

    fn think(&self) {
        self.thoughts
            .send(format!("New idea from {}!", &self.name))
            .unwrap();
    }

    fn eat(&self) {
        loop {
            if self.left_fork.try_lock().is_ok() && self.right_fork.try_lock().is_ok() {
                break;
            }
            println!("{} waiting to eat...", &self.name);
            thread::sleep(Duration::from_millis(10));
        }
        //println!("{} eating...", &self.name);
        thread::sleep(Duration::from_millis(10));
    }
}

static PHILOSOPHERS: &[&str] =
    &["Socrates", "Hypatia", "Plato", "Aristotle", "Pythagoras"];

fn main() {
    let phil_count = PHILOSOPHERS.len();
    let forks = vec![Arc::new(Mutex::new(Fork)); phil_count];
    let (sender, receiver) = mpsc::channel();

    let mut philosophers = Vec::new();
    for i in 0..phil_count {
        let left_fork = Arc::clone(&forks[i]);
        let right_fork = Arc::clone(&forks[(i + 1) % phil_count]);
        philosophers.push(Philosopher::new(PHILOSOPHERS[i], left_fork, right_fork, sender.clone()));
    }

    let mut handles = Vec::new();
    for phil in philosophers {
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                phil.eat();
                phil.think();
            }
        }));
    }
    drop(sender);
    handles.into_iter().for_each(|h| h.join().unwrap());
    for thought in receiver.iter() {
        println!("{thought}");
    }
}