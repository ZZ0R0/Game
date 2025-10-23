
/* -------------------- Bounds -------------------- */

#[derive(Debug, Clone, Copy)]
pub struct RectBoundaries {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
    pub z_min: i32,
    pub z_max: i32,
}
impl RectBoundaries {
    pub fn null() -> Self {
        Self {
            x_min: 0,
            x_max: 0,
            y_min: 0,
            y_max: 0,
            z_min: 0,
            z_max: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CircleBoundaries {
    pub radius: f32,
}
impl CircleBoundaries {
    pub fn null() -> Self {
        Self { radius: 0.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Boundaries {
    Rect(RectBoundaries),
    Circle(CircleBoundaries),
}
impl Boundaries {
    pub fn kind(&self) -> &'static str {
        match self {
            Boundaries::Rect(_) => "Rectangular",
            Boundaries::Circle(_) => "Circular",
        }
    }
}