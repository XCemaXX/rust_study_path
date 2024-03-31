use super::Widget;

pub struct Window {
    title: String,
    widgets: Vec<Box<dyn Widget>>,
}

fn draw_into_center(str: &str, buffer: &mut dyn std::fmt::Write, width: usize) {
    let _ = buffer.write_str(&format!("{:^width$}", str));
}

impl Window {
    pub fn new(title: &str) -> Window {
        Window { title: title.to_owned(), widgets: Vec::new() }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    fn inner_width(&self) -> usize {
        std::cmp::max(
            self.title.chars().count(),
            self.widgets.iter().map(|w| w.width()).max().unwrap_or(0),
        )
    }

    fn draw_title(&self, buffer: &mut dyn std::fmt::Write) {
        let width = self.width();
        let _ = buffer.write_str(&format!("={:=<width$}=\n", ""));
        for line in self.title.lines() {
            let _ = buffer.write_str("|");
            draw_into_center(line, buffer, width);
            let _ = buffer.write_str("|\n");
        }
        let _ = buffer.write_str(&format!("={:=<width$}=\n", ""));
    }

    fn draw_widgets(&self, buffer: &mut dyn std::fmt::Write) {
        let width = self.width();
        let mut inner = String::new();
        for w in self.widgets.iter() {
            (*w).draw_into(&mut inner);
        }
        for line in inner.lines() {
            let _ = buffer.write_str(&format!("|{: <width$}|\n", line));
        }
        let _ = buffer.write_str(&format!("={:=<width$}=\n", ""));
    }
}

impl Widget for Window {
    fn width(&self) -> usize {
        self.inner_width() + 2
    }

    // Draw/write widget into buffer
    fn draw_into(&self, buffer: &mut dyn std::fmt::Write) {
        self.draw_title(buffer);
        self.draw_widgets(buffer);
    }
}