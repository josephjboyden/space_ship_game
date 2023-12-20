use std::mem;

use bevy::prelude::*;

use super::quad_tree::{QuadTree, AABB};

use num::FromPrimitive;
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
}

impl CircleCollider {
    pub fn new(radius: f32) -> Self {
        Self { radius: radius }
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

#[derive(FromPrimitive, Clone, Copy)]
pub enum CollisionLayerNames {
    Alien,
    Ship,
    Projectile,
    HealthPack,
}

pub struct CollisionLayer {
    id: usize, //id must match index in collision layer reasorces "layers" vec
    collides_with: Vec<usize>,
    pub in_layer: Vec<Entity>,
}

impl CollisionLayer {
    fn new(id: usize, collides_with: Vec<usize>) -> Self {
        Self {
            id: id,
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
                CollisionLayer::new(CollisionLayerNames::Alien as usize, vec![]),
                CollisionLayer::new(
                    CollisionLayerNames::Ship as usize,
                    vec![CollisionLayerNames::Alien as usize],
                ),
                CollisionLayer::new(
                    CollisionLayerNames::Projectile as usize,
                    vec![CollisionLayerNames::Alien as usize],
                ),
                CollisionLayer::new(
                    CollisionLayerNames::HealthPack as usize,
                    vec![CollisionLayerNames::Ship as usize],
                ),
            ],
        }
    }
}
impl CollisionLayers {
    fn add_collision_layer(&mut self, collides_with: Vec<usize>) -> usize {
        let collision_layer = CollisionLayer::new(self.layers.len(), collides_with);
        let id = collision_layer.id;
        self.layers.push(collision_layer);
        id
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
        for entity in &layer.in_layer {
            if let Ok((circle_collider, transform)) = circle_collider_query.get(*entity) {
                handle_circle_collisions(
                    circle_collider,
                    transform.translation.xy(),
                    &entity,
                    &circle_collider_query,
                    &quad_tree,
                    &mut collide_event_writer,
                );
            }
        }
    }
}

const COLLIDER_CHECK_DISTANCE: f32 = 100.; //must be larger than max radius of the largest collider
fn handle_circle_collisions(
    a: &CircleCollider,
    a_translation: Vec2,
    a_entity: &Entity,
    circle_collider_query: &Query<(&CircleCollider, &Transform)>,
    quad_tree: &Res<QuadTree>,
    collide_event_writer: &mut EventWriter<CollideEvent>,
) {
    for b_entity in quad_tree.query_range(&AABB::new(
        a_translation,
        a.radius + COLLIDER_CHECK_DISTANCE,
    )) {
        if let Ok((b, b_transform)) = circle_collider_query.get(b_entity) {
            if circle_circle_collision(a, a_translation, b, b_transform.translation.xy()) {
                collide_event_writer.send(CollideEvent::new(*a_entity, b_entity))
            }
        }
    }
}
