mod gui_widgets;

use gui_widgets::Widget;
use gui_widgets::{Window, Label, Button};

fn main() {
    let mut window = Window::new("Rust GUI Demo\n1.23");
    window.add_widget(Box::new(Label::new("This is a small text GUI demo.")));
    window.add_widget(Box::new(Button::new("Click me!\nSecond line")));
    window.add_widget(Box::new(Label::new("1\n2\n3")));
    window.add_widget(Box::new(Button::new("LOOOOOOOOOOOOOOOOOOOOOOONG BUUUUUUUUUUUUUUTTTTTTTON")));
    window.draw();
}