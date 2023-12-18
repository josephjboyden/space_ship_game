use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app .add_systems(Update, (
                acceleration_physics_update, 
                velocity_physics_update,
                //draw_colliders
            ));
    }
}

#[derive(Component)]
pub struct Physics;

impl Default for Physics {
    fn default() -> Self {
        Physics
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

fn acceleration_physics_update(
    mut physics_query: Query<(&mut Transform, &mut Velocity, &Acceleration), With<Physics>>,
    time: Res<Time>
) {
    for (mut transform, mut velocity, acceleration) in physics_query.iter_mut()
    {
        let delta_time = time.delta_seconds();

        let mut a = acceleration.value.clone();
        if acceleration.local {
            a = transform.rotation.mul_vec3(Vec3 { x: a.x, y: a.y, z: 0. }).xy();
        }

        let s = velocity.0 * delta_time + 0.5 * a * delta_time * delta_time;
        velocity.0 += a * delta_time;

        transform.translation += Vec3 {x: s.x, y: s.y, z: 0.};
    }
}

fn velocity_physics_update(
    mut physics_query: Query<(&mut Transform, &Velocity), (With<Physics>, Without<Acceleration>)>,
    time: Res<Time>
) {
    for (mut transform, velocity) in physics_query.iter_mut() {
        transform.translation += Vec3::new(velocity.0.x, velocity.0.y , 0.) * time.delta_seconds();
    }
}

#[derive(Component)]
pub struct CircleCollider {
    radius: f32,
}

impl CircleCollider {
    pub fn new(radius: f32) -> Self {
        Self {radius: radius}
    }
}

pub fn circle_circle_collision (a: &CircleCollider, a_translation: Vec2, b: &CircleCollider, b_translation: Vec2) -> bool {
    (a_translation - b_translation).length() < a.radius + b.radius
}

// fn draw_colliders(
//     mut gizmos: Gizmos,
//     circle_collider_query: Query<(&Transform, &CircleCollider)>
// ) {
//     for (transform, collider) in circle_collider_query.iter() {
//         gizmos.circle_2d(transform.translation.xy(), collider.radius, Color::RED);
//     }
// }