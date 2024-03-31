use super::{Label, Widget};

pub struct Button {
    label: Label,
}

impl Button {
    pub fn new(label: &str) -> Button {
        Button { label: Label::new(label) }
    }
}

fn draw_into_center(str: &str, buffer: &mut dyn std::fmt::Write, width: usize) {
    let _ = buffer.write_str(&format!("{:^width$}", str));
}

impl Widget for Button {
    fn width(&self) -> usize {
        self.label.width() + 2
    }

    // Draw/write widget into buffer
    fn draw_into(&self, buffer: &mut dyn std::fmt::Write) {
        let width = self.width();
        let _ = buffer.write_str(&format!("+{:-<width$}+\n", ""));
        for line in self.label.get_text().lines() {
            let _ = buffer.write_str("|");
            draw_into_center(line, buffer, width);
            let _ = buffer.write_str("|\n");
        }
        let _ = buffer.write_str(&format!("+{:-<width$}+\n", ""));
    }
}
