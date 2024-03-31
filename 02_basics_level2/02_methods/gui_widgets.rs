mod button;
mod label;
mod window;

pub trait Widget {
    fn width(&self) -> usize;

    // Draw/write widget into buffer
    fn draw_into(&self, buffer: &mut dyn std::fmt::Write);

    // Draw into stdout
    fn draw(&self) {
        let mut buffer = String::new();
        self.draw_into(&mut buffer);
        println!("{buffer}");
    }
}

pub use button::Button;
pub use label::Label;
pub use window::Window;