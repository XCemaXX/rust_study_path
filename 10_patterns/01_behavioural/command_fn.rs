type FnPtr = fn() -> &'static str;

struct Step {
    execute: FnPtr,
    cleanup: FnPtr,
}

struct Pipeline {
    steps: Vec<Step>,
}

impl Pipeline {
    fn new() -> Self {
        Self { steps: Vec::new() }
    }
    fn add_step(&mut self, execute: FnPtr, cleanup: FnPtr) {
        self.steps.push(Step { execute, cleanup });
    }
    fn execute(&mut self) -> Vec<&str> {
        self.steps
            .iter()
            .map(|step| (step.execute)())
            .collect::<Vec<_>>()
    }
    fn cleanup(&self) -> Vec<&str> {
        self.steps
            .iter()
            .rev()
            .map(|step| (step.cleanup)())
            .collect::<Vec<_>>()
    }
}

fn download() -> &'static str {
    "Downloaded"
}

fn cleanup_download() -> &'static str {
    "Remove downloaded data"
}

fn main() {
    let mut p = Pipeline::new();
    p.add_step(download, cleanup_download);
    p.add_step(|| "Builded", || "Remove builded artifacts");
    println!("{:?}", p.execute());
    println!("{:?}", p.cleanup());
}
