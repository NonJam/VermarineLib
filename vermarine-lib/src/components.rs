//
// Transform

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            x: 0f64,
            y: 0f64,
            r: 0f64,
        }
    }
}

impl Transform {
    pub fn new(x: f64, y: f64, r: f64) -> Self {
        Transform {
            x,
            y,
            r,
            ..Transform::default()
        }
    }

    pub fn get_angle_to(&self, x: f64, y: f64) -> f64 {
        let result = (self.y - y)
            .to_radians()
            .atan2((self.x - x).to_radians())
            .to_degrees();
        if result < 0f64 {
            (result + 630f64) % 360f64
        } else {
            (result + 270f64) % 360f64
        }
    }

    pub fn collides_with(&self, other: Transform) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        (dx * dx + dy * dy).sqrt() < self.r + other.r
    }
}