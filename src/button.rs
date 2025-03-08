pub struct Button {
    pub name: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: [u8; 4],
    default_color: [u8; 4],
    clicked_color: [u8; 4],
}

impl Button {
    pub fn new(
        name: String,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        default_color: [u8; 4],
        clicked_color: [u8; 4],
    ) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
            color: default_color,
            default_color,
            clicked_color,
        }
    }

    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x as f64
            && px <= (self.x + self.width) as f64
            && py >= self.y as f64
            && py <= (self.y + self.height) as f64
    }

    pub fn click(&mut self) {
        if self.color == self.default_color {
            self.color = self.clicked_color;
        } else {
            self.color = self.default_color;
        }
        println!("click {}", self.name)
    }

    pub fn draw(&self, canvas: &mut [u8], stride: i32) {
        for y in self.y..self.y + self.height {
            for x in self.x..self.x + self.width {
                let offset = (y * stride + x * 4) as usize;
                canvas[offset..offset + 4].copy_from_slice(&self.color);
            }
        }
    }
}
