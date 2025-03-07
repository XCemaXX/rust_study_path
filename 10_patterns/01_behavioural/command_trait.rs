pub trait Step {
    fn execute(&mut self, prev_res: bool) -> Result<&str, &str>;
    fn cleanup(&self) -> &str;
}

struct DownloadStep {
    download_status: bool,
}
impl Step for DownloadStep {
    fn execute(&mut self, prev_res: bool) -> Result<&str, &str> {
        match (prev_res, self.download_status) {
            (true, true) => Ok("Downloaded"),
            (true, false) => Err("Failed to download"),
            (false, _) => {
                self.download_status = false;
                Err("Skipped")
            }
        }
    }

    fn cleanup(&self) -> &str {
        if self.download_status {
            "Remove downloaded data"
        } else {
            "Nothing to remove"
        }
    }
}
struct BuildStep {
    build_status: bool,
}
impl Step for BuildStep {
    fn execute(&mut self, prev_res: bool) -> Result<&str, &str> {
        match (prev_res, self.build_status) {
            (true, true) => Ok("Builded"),
            (true, false) => Err("Failed to build"),
            (false, _) => {
                self.build_status = false;
                Err("Skipped")
            }
        }
    }

    fn cleanup(&self) -> &str {
        if self.build_status {
            "Remove builded artifacts"
        } else {
            "Nothing to remove"
        }
    }
}

struct Pipeline {
    steps: Vec<Box<dyn Step>>,
}

impl Pipeline {
    fn new() -> Self {
        Self { steps: Vec::new() }
    }
    fn add_step(&mut self, step: Box<dyn Step>) {
        self.steps.push(step);
    }
    fn execute(&mut self) -> bool {
        self.steps
            .iter_mut()
            .fold(true, |prev_res, step| match step.execute(prev_res) {
                Ok(msg) => {
                    println!("{msg}");
                    true
                }
                Err(msg) => {
                    println!("{msg}");
                    false
                }
            })
    }
    fn cleanup(&self) -> Vec<&str> {
        self.steps
            .iter()
            .rev()
            .map(|step| step.cleanup())
            .collect::<Vec<_>>()
    }
}

fn main() {
    let mut p = Pipeline::new();
    p.add_step(Box::new(DownloadStep {
        download_status: true,
    }));
    p.add_step(Box::new(BuildStep { build_status: true }));
    assert!(p.execute());
    println!("{:?}", p.cleanup());

    let mut p = Pipeline::new();
    p.add_step(Box::new(DownloadStep {
        download_status: false,
    }));
    p.add_step(Box::new(BuildStep { build_status: true }));
    assert!(!p.execute());
    println!("{:?}", p.cleanup());

    let mut p = Pipeline::new();
    p.add_step(Box::new(DownloadStep {
        download_status: true,
    }));
    p.add_step(Box::new(BuildStep {
        build_status: false,
    }));
    assert!(!p.execute());
    println!("{:?}", p.cleanup());
}
