trait StrategyInterface {
    fn make_step_one(&self, s: &str) -> String;
    fn make_step_two(&self) -> usize;
}

struct MainAlgorithm;

impl MainAlgorithm {
    fn execute(&self, si: impl StrategyInterface) -> String {
        let s = "Test ";
        let r1 = si.make_step_one(s);
        let r2 = si.make_step_two();
        r1 + &r2.to_string()
    }
}

struct MyImplementation {
    data: String,
    i: usize,
}

impl StrategyInterface for MyImplementation {
    fn make_step_one(&self, s: &str) -> String {
        s.to_owned() + &self.data
    }

    fn make_step_two(&self) -> usize {
        self.i
    }
}

fn main() {
    let i = MyImplementation {
        data: "hi".to_string(),
        i: 23,
    };
    let a = MainAlgorithm.execute(i);
    println!("{}", a);

    let v = vec![1, 2, 3];
    let r = v.iter().map(|e| e * 20).collect::<Vec<_>>();
    println!("Strategy closures multiply by 20: {:?}", r);
}
