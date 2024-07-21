#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct RGBf32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RGBf32 {
    pub const BLACK: RGBf32 = RGBf32::new(0.0, 0.0, 0.0);
    pub const WHITE: RGBf32 = RGBf32::new(1.0, 1.0, 1.0);
    pub const RED: RGBf32 = RGBf32::new(1.0, 0.0, 0.0);
    pub const GREEN: RGBf32 = RGBf32::new(0.0, 1.0, 0.0);
    pub const BLUE: RGBf32 = RGBf32::new(0.0, 0.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn lerp(&self, o: RGBf32, w: f32) -> RGBf32 {
        let x = 1.0 - w;

        RGBf32 {
            r: (self.r * w) + (o.r * x),
            g: (self.g * w) + (o.g * x),
            b: (self.b * w) + (o.b * x),
        }
    }
}
