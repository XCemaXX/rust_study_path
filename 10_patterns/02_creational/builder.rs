pub struct Builder {
    aspect_ratio: f32,
    image_width: usize,
}

#[derive(Debug)]
pub struct Image {
    width: usize,
    height: usize,
}

impl Image {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
        }
    }

    pub fn aspect_ratio(mut self, x: f32) -> Self {
        self.aspect_ratio = x;
        self
    }
    pub fn image_width(mut self, x: usize) -> Self {
        self.image_width = x;
        self
    }
    pub fn build(self) -> Image {
        let image_height = self.image_width as f32 / self.aspect_ratio;
        let image_height = if image_height < 1.0 {
            1
        } else {
            image_height as usize
        };
        Image {
            width: self.image_width,
            height: image_height,
        }
    }
}

fn main() {
    let i = Image::builder().image_width(300).build();
    println!("{i:?}");
}
