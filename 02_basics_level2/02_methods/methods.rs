
struct Rase {
    name: String,
    laps: Vec<i32>,
}

impl Rase {
    // static
    fn new(name: &str) -> Self { // Self is synonym for Race
        Self { name: String::from(name), laps: Vec::new() }
    }

    // exclusive borrowing. Possible to change Struct
    fn add_lap(&mut self, lap: i32) {
        self.laps.push(lap);
    }

    // only read. Const method
    fn print_laps(&self) {
        for (i, lap) in self.laps.iter().enumerate() {
            println!("Lap {i}: {lap} sec");
        }
    }

    // exclusive ownership. Destructor
    fn finish(self) {
        let total_secs: i32 = self.laps.iter().sum();
        println!("Rase {} ended, total time: {total_secs} s", self.name);
    }
}

fn main() {
    let mut race = Rase::new("Monaco Grand Prix");
    race.add_lap(70);
    race.add_lap(68);
    race.print_laps();
    race.add_lap(71);
    race.print_laps();
    race.finish();
}