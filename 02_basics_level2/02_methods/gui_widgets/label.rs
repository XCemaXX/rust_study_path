use super::Widget;

pub struct Label {
    label: String, // can be several strings
}

impl Label {
    // Constructor
    pub fn new(label: &str) -> Self {
        Label { label: label.to_owned() }
    }

    pub fn get_text(&self) -> &str {
        &self.label
    }
}

impl Widget for Label {
    fn width(&self) -> usize {
        self.label.len() + 2
    }

    // Draw/write widget into buffer
    fn draw_into(&self, buffer: &mut dyn std::fmt::Write) {
        let _ = buffer.write_str(&format!("{}\n", self.label));
    }
}