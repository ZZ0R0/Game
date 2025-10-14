#[derive(Debug, Clone)]
pub struct Volume {
    pub points: Vec<(f32, f32, f32)>,
    pub position: (f32, f32, f32),
}

impl Volume {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            position: (0.0, 0.0, 0.0),
        }
    }

    pub fn unit_cube() -> Self {
        Self {
            points: vec![
                (-0.5, -0.5, -0.5), (0.5, -0.5, -0.5),
                (0.5, 0.5, -0.5), (-0.5, 0.5, -0.5),
                (-0.5, -0.5, 0.5), (0.5, -0.5, 0.5),
                (0.5, 0.5, 0.5), (-0.5, 0.5, 0.5),
            ],
            position: (0.0, 0.0, 0.0),
        }
    }

    pub fn block_volume() -> Self {
        // A standard block size for Space Engineers-like games
        Self {
            points: vec![
                (-1.25, -1.25, -1.25), (1.25, -1.25, -1.25),
                (1.25, 1.25, -1.25), (-1.25, 1.25, -1.25),
                (-1.25, -1.25, 1.25), (1.25, -1.25, 1.25),
                (1.25, 1.25, 1.25), (-1.25, 1.25, 1.25),
            ],
            position: (0.0, 0.0, 0.0),
        }
    }
}