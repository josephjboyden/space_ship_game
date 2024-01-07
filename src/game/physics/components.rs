use super::collision::CollisionLayerNames;
use bevy::prelude::*;

#[derive(Component)]
pub struct Physics {
    pub use_collisions: bool,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            use_collisions: false,
        }
    }
}

impl Physics {
    pub fn new(use_collisions: bool) -> Self {
        Self {
            use_collisions: use_collisions,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

impl Default for Velocity {
    fn default() -> Self {
        Velocity(Vec2::default())
    }
}

#[derive(Component)]
pub struct Acceleration {
    pub value: Vec2,
    pub local: bool,
}

impl Default for Acceleration {
    fn default() -> Self {
        Self {
            value: Vec2::ZERO,
            local: false,
        }
    }
}

#[derive(Component, Default)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32,
    pub layer: CollisionLayerNames,
}

impl CircleCollider {
    pub fn new(radius: f32, layer: CollisionLayerNames) -> Self {
        Self {
            radius: radius,
            layer: layer,
        }
    }
}

#[derive(Component)]
pub struct AARectCollider {
    pub size: Vec2,
    half_size: Vec2,
    pub layer: CollisionLayerNames,
}

impl AARectCollider {
    pub fn new(size: Vec2, layer: CollisionLayerNames) -> Self {
        Self {
            size: size,
            half_size: size / 2.,
            layer: layer,
        }
    }

    pub fn dist(&self, point: Vec2, translation: Vec2) -> f32 {
        let tl = translation - self.half_size;
        let br = translation + self.half_size;
        let d = (tl - point).max(point - br);
        (Vec2::ZERO.max(d)).length() + (d.x.max(d.y)).min(0.)
    }

    pub fn nearest_point(&self, point: Vec2, translation: Vec2) -> (Vec2, bool) {
        let tl = translation - self.half_size;
        let br = translation + self.half_size;
        let inside_x = tl.x < point.x && point.x < br.x;
        let inside_y = tl.y < point.y && point.y < br.y;
        let point_inside_rectangle = inside_x && inside_y;

        if !point_inside_rectangle {
            return (
                Vec2::new(tl.x.max(point.x.min(br.x)), tl.y.max(point.y.min(br.y))),
                false,
            );
        } else {
            let distance_to_positive_bounds = br - point;
            let distance_to_negative_bounds = tl - point;
            let smallest_x = distance_to_positive_bounds
                .x
                .min(distance_to_negative_bounds.x);
            let smallest_y = distance_to_positive_bounds
                .y
                .min(distance_to_negative_bounds.y);
            let smallest_distance = smallest_x.min(smallest_y);

            if smallest_distance == distance_to_positive_bounds.x {
                return (Vec2::new(br.x, point.y), true);
            } else if smallest_distance == distance_to_negative_bounds.x {
                return (Vec2::new(tl.x, point.y), true);
            } else if smallest_distance == distance_to_positive_bounds.y {
                return (Vec2::new(point.x, br.y), true);
            } else {
                return (Vec2::new(point.x, tl.y), true);
            }
        }
    }
}
