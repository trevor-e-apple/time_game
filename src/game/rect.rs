use cgmath::Vector2;

#[derive(Clone, Copy)]
pub struct Rect {
    pub bottom_left: Vector2<f32>,
    pub dim: Vector2<f32>,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            bottom_left: Vector2 { x: 0.0, y: 0.0 },
            dim: Vector2 { x: 0.0, y: 0.0 },
        }
    }
}

impl Rect {
    pub fn with_center(center: Vector2<f32>, dim: Vector2<f32>) -> Self {
        Self {
            bottom_left: center - (dim / 2.0),
            dim,
        }
    }

    pub fn get_center(&self) -> Vector2<f32> {
        self.bottom_left + (self.dim / 2.0)
    }

    pub fn set_center(&mut self, center: Vector2<f32>) {
        self.bottom_left = center - (self.dim / 2.0);
    }

    pub fn point_in(&self, point: &Vector2<f32>) -> bool {
        point.x > self.bottom_left.x
            && point.x < self.bottom_left.x + self.dim.x
            && point.y > self.bottom_left.y
            && point.y < self.bottom_left.y + self.dim.y
    }
}
