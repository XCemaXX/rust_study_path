#![allow(dead_code)]

#[derive(Debug)]
enum Event {
    Arrived(Floor),
    Opened,
    Closed,
    ButtonPressed(Button)
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
}

#[derive(Debug)]
enum Button {
    LobbyButton { floor: Floor, direction: Direction },
    LiftButton(Floor)
}

#[derive(Debug)]
struct AgeWeight(i32, i32);

#[derive(Debug)]
struct Person {
    name: String,
    age_weight: AgeWeight,
}

type Floor = i32;

fn car_arrived(floor: i32) -> Event {
    Event::Arrived(floor)
}

fn car_door_opened() -> Event {
    Event::Opened
}

fn car_door_closed() -> Event {
    Event::Closed
}

fn lobby_call_button_pressed(floor: i32, dir: Direction) -> Event {
    Event::ButtonPressed(Button::LobbyButton{ floor: floor, direction: dir})
}

fn car_floor_button_pressed(floor: i32) -> Event {
    Event::ButtonPressed(Button::LiftButton(floor))
}

fn main() {
    let man = Person{ name: "Kate".to_string(), age_weight: AgeWeight(10, 30) };
    println!(
        "{:?} on the first floor pressed the up button: {:?}", man,
        lobby_call_button_pressed(0, Direction::Up)
    );
    println!("The elevator arrived on the first floor: {:?}", car_arrived(0));
    println!("The elevator doors opened: {:?}", car_door_opened());
    println!(
        "Man pressed the button for the third floor: {:?}",
        car_floor_button_pressed(3)
    );
    println!("The elevator doors closed: {:?}", car_door_closed());
    println!("The elevator arrived on the third floor: {:?}", car_arrived(3));
}