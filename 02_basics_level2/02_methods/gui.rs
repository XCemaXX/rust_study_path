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

pub struct Label {
    label: String, // can be several strings
}

fn draw_into_center(str: &str, buffer: &mut dyn std::fmt::Write, width: usize) {
    let _ = buffer.write_str(&format!("{:^width$}", str));
}

impl Label {
    // Constructor
    fn new(label: &str) -> Label {
        Label { label: label.to_owned() }
    }
}

pub struct Button {
    label: Label,
}

impl Button {
    fn new(label: &str) -> Button {
        Button { label: Label::new(label) }
    }
}

pub struct Window {
    title: String,
    widgets: Vec<Box<dyn Widget>>,
}

impl Window {
    fn new(title: &str) -> Window {
        Window { title: title.to_owned(), widgets: Vec::new() }
    }

    fn add_widget(&mut self, widget: Box<dyn Widget>) {
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
impl Widget for Button {
    fn width(&self) -> usize {
        self.label.width() + 2
    }

    // Draw/write widget into buffer
    fn draw_into(&self, buffer: &mut dyn std::fmt::Write) {
        let width = self.width();
        let _ = buffer.write_str(&format!("+{:-<width$}+\n", ""));
        for line in self.label.label.lines() {
            let _ = buffer.write_str("|");
            draw_into_center(line, buffer, width);
            let _ = buffer.write_str("|\n");
        }
        let _ = buffer.write_str(&format!("+{:-<width$}+\n", ""));
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

fn main() {
    let mut window = Window::new("Rust GUI Demo\n1.23");
    window.add_widget(Box::new(Label::new("This is a small text GUI demo.")));
    window.add_widget(Box::new(Button::new("Click me!\nSecond line")));
    window.add_widget(Box::new(Label::new("1\n2\n3")));
    window.add_widget(Box::new(Button::new("LOOOOOOOOOOOOOOOOOOOOOOONG BUUUUUUUUUUUUUUTTTTTTTON")));
    window.draw();
}