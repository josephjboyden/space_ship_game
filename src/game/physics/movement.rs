use super::components::{Acceleration, Mass, Physics, Velocity};
use bevy::prelude::*;

pub fn acceleration_physics_update(
    mut physics_query: Query<(&mut Transform, &mut Velocity, &Acceleration), With<Physics>>,
    time: Res<Time<Fixed>>,
) {
    let delta_time = time.delta_seconds();
    for (mut transform, mut velocity, acceleration) in physics_query.iter_mut() {
        let mut a = acceleration.value.clone();
        if acceleration.local {
            a = transform
                .rotation
                .mul_vec3(Vec3 {
                    x: a.x,
                    y: a.y,
                    z: 0.,
                })
                .xy();
        }

        let s = velocity.0 * delta_time + 0.5 * a * delta_time * delta_time;
        velocity.0 += a * delta_time;

        transform.translation += Vec3 {
            x: s.x,
            y: s.y,
            z: 0.,
        };
    }
}

pub fn velocity_physics_update(
    mut physics_query: Query<(&mut Transform, &Velocity), (With<Physics>, Without<Acceleration>)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in physics_query.iter_mut() {
        transform.translation += Vec3::new(velocity.0.x, velocity.0.y, 0.) * time.delta_seconds();
    }
}

#[derive(Event)]
pub struct AddImpulseEvent {
    value: Vec2,
    entity: Entity,
}

impl AddImpulseEvent {
    pub fn new(change_in_velocity: Vec2, mass: f32, apply_to_entity: Entity) -> Self {
        Self {
            value: change_in_velocity * mass,
            entity: apply_to_entity,
        }
    }
}

pub fn apply_impulse(
    mut add_impulse_event_reader: EventReader<AddImpulseEvent>,
    mut physics_query: Query<(&Mass, &mut Velocity), With<Physics>>,
) {
    for event in add_impulse_event_reader.read() {
        if let Ok((mass, mut velocity)) = physics_query.get_mut(event.entity) {
            velocity.0 += event.value / mass.0
        }
    }
}
