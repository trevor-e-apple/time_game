use cgmath::Vector2;

pub struct Rect {
    pub bottom_left: Vector2<f32>,
    pub dim: Vector2<f32>,
}

impl Rect {
    pub fn with_center(center: Vector2<f32>, dim: Vector2<f32>) -> Self {
        Self {
            bottom_left: center - dim,
            dim,
        }
    }

    pub fn get_center(&self) -> Vector2<f32> {
        todo!()
    }

    pub fn point_in(&self, point: &Vector2<f32>) -> bool {
        point.x > self.bottom_left.x
            && point.x < self.bottom_left.x + self.dim.x
            && point.y > self.bottom_left.y
            && point.y < self.bottom_left.y + self.dim.y
    }
}
