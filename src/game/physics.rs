use bevy::prelude::*;

use super::quad_tree::{QuadTree, AABB};
use num_derive::FromPrimitive;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollideEvent>()
            .insert_resource(CollisionLayers::default())
            .add_systems(
                Update,
                (
                    acceleration_physics_update,
                    velocity_physics_update,
                    handle_collisions,
                    //draw_colliders
                ),
            );
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
    time: Res<Time>,
) {
    for (mut transform, mut velocity, acceleration) in physics_query.iter_mut() {
        let delta_time = time.delta_seconds();

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

fn velocity_physics_update(
    mut physics_query: Query<(&mut Transform, &Velocity), (With<Physics>, Without<Acceleration>)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in physics_query.iter_mut() {
        transform.translation += Vec3::new(velocity.0.x, velocity.0.y, 0.) * time.delta_seconds();
    }
}

#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32,
    layer: CollisionLayerNames,
}

impl CircleCollider {
    pub fn new(radius: f32, layer: CollisionLayerNames) -> Self {
        Self {
            radius: radius,
            layer: layer,
        }
    }
}

fn circle_circle_collision(
    a: &CircleCollider,
    a_translation: Vec2,
    b: &CircleCollider,
    b_translation: Vec2,
) -> bool {
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

#[derive(FromPrimitive, Clone, Copy, PartialEq, Eq)]
pub enum CollisionLayerNames {
    CollidesWithAliens,
    Ship,
    Aliens,
    HealthPacks,
}

pub struct CollisionLayer {
    collides_with: Vec<CollisionLayerNames>,
    pub in_layer: Vec<Entity>,
}

impl CollisionLayer {
    fn new(collides_with: Vec<CollisionLayerNames>) -> Self {
        Self {
            collides_with: collides_with,
            in_layer: vec![],
        }
    }
}

#[derive(Resource)]
pub struct CollisionLayers {
    pub layers: Vec<CollisionLayer>,
}
impl Default for CollisionLayers {
    fn default() -> Self {
        Self {
            layers: vec![
                CollisionLayer::new(vec![CollisionLayerNames::Aliens]), //match enum order
                CollisionLayer::new(vec![
                    CollisionLayerNames::HealthPacks,
                    CollisionLayerNames::Aliens,
                ]),
                CollisionLayer::new(vec![]),
                CollisionLayer::new(vec![]),
            ],
        }
    }
}

#[derive(Event)]
pub struct CollideEvent {
    pub a: Entity,
    pub b: Entity,
}

impl CollideEvent {
    fn new(a: Entity, b: Entity) -> Self {
        Self { a: a, b: b }
    }
}

fn handle_collisions(
    circle_collider_query: Query<(&CircleCollider, &Transform)>,
    collision_layers: Res<CollisionLayers>,
    quad_tree: Res<QuadTree>,
    mut collide_event_writer: EventWriter<CollideEvent>,
) {
    for layer in &collision_layers.layers {
        if layer.collides_with.len() == 0 {
            continue;
        }

        for entity in &layer.in_layer {
            if let Ok((circle_collider, transform)) = circle_collider_query.get(*entity) {
                handle_circle_collisions(
                    circle_collider,
                    transform.translation.xy(),
                    &entity,
                    &circle_collider_query,
                    &quad_tree,
                    &mut collide_event_writer,
                    &layer.collides_with,
                );
            }
        }
    }
}

const COLLIDER_CHECK_DISTANCE: f32 = 30.; //must be larger than max radius of the largest collider
fn handle_circle_collisions(
    a: &CircleCollider,
    a_translation: Vec2,
    a_entity: &Entity,
    circle_collider_query: &Query<(&CircleCollider, &Transform)>,
    quad_tree: &Res<QuadTree>,
    collide_event_writer: &mut EventWriter<CollideEvent>,
    collides_with: &Vec<CollisionLayerNames>,
) {
    for b_entity in quad_tree.query_range(&AABB::new(
        a_translation,
        a.radius + COLLIDER_CHECK_DISTANCE,
    )) {
        if let Ok((b, b_transform)) = circle_collider_query.get(b_entity) {
            if collides_with.contains(&b.layer) {
                if circle_circle_collision(a, a_translation, b, b_transform.translation.xy()) {
                    collide_event_writer.send(CollideEvent::new(*a_entity, b_entity))
                }
            }
        }
    }
}
