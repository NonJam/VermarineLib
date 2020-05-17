pub struct SnakeGame {
    pub length: usize,
    pub skip_frames: usize,
    pub frame_counter: usize,
    pub move_x: f64,
    pub move_y: f64,
    pub move_x_prev: f64,
    pub move_y_prev: f64
}

impl SnakeGame {
    pub fn new() -> Self {
        SnakeGame {
            length: 3,
            skip_frames: 2,
            frame_counter: 0,
            move_x_prev: 32f64,
            move_y_prev: 0f64,
            move_x : 32f64,
            move_y: 0f64
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub position: usize
}