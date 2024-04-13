use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use tokio::time;

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

    async fn think(&self) {
        self.thoughts
            .send(format!("New idea from {}!", &self.name))
            .await
            .unwrap();
    }

    async fn eat(&self) {
        println!("{} eating...", &self.name);
        time::sleep(time::Duration::from_millis(5)).await;
    }
}

static PHILOSOPHERS: &[&str] =
     &["Socrates", "Hypatia", "Plato", "Aristotle", "Pythagoras"];

#[tokio::main]
async fn main() {
    let phil_count = PHILOSOPHERS.len();
    let forks = vec![Arc::new(Mutex::new(Fork)); phil_count];
    
    let (philosophers, mut receiver) = {
        let mut philosophers = Vec::new();
        let (sender, receiver) = mpsc::channel(10);
        for i in 0..phil_count {
            let left_fork = Arc::clone(&forks[i]);
            let right_fork = Arc::clone(&forks[(i + 1) % phil_count]);
            philosophers.push(Philosopher::new(PHILOSOPHERS[i], left_fork, right_fork, sender.clone()));
        }
        (philosophers, receiver)
    };

    for phil in philosophers {
        tokio::spawn(async move {
            for _ in 0..100 {
                phil.eat().await;
                phil.think().await;
            }
        });
    }

    while let Some(thought) = receiver.recv().await {
        println!("{thought}");
    }
}