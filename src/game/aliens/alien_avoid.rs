use bevy::prelude::*;

#[derive(Component)]
pub struct AARectAlienAvoid {
    pub half_size: Vec2,
}

impl AARectAlienAvoid {
    pub fn dist(&self, point: Vec3, translation: Vec3) -> f32 {
        let tl = translation.xy() - self.half_size;
        let br = translation.xy() + self.half_size;
        let d = (tl - point.xy()).max(point.xy() - br);
        (Vec2::ZERO.max(d)).length() + d.x.max(d.y.min(0.))
    }
}
