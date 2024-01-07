use super::super::quad_tree::{QuadTree, AABB};
use super::components::{AARectCollider, CircleCollider, Physics, Velocity};
use bevy::prelude::*;
use num_derive::FromPrimitive;

fn circle_circle_collision(
    a: &CircleCollider,
    a_translation: Vec2,
    b: &CircleCollider,
    b_translation: Vec2,
) -> bool {
    (a_translation - b_translation).length() <= a.radius + b.radius
}

fn circle_aa_rect_collision(
    a: &CircleCollider,
    a_translation: Vec2,
    b: &AARectCollider,
    b_translation: Vec2,
) -> bool {
    b.dist(a_translation, b_translation) <= a.radius
}

#[derive(FromPrimitive, Clone, Copy, PartialEq, Eq)]
pub enum CollisionLayerNames {
    CollidesWithAliens,
    Ship,
    Aliens,
    HealthPacks,
    Walls,
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
                CollisionLayer::new(vec![CollisionLayerNames::Aliens]), //match enum order and length
                CollisionLayer::new(vec![
                    CollisionLayerNames::HealthPacks,
                    CollisionLayerNames::Aliens,
                    CollisionLayerNames::Walls,
                ]),
                CollisionLayer::new(vec![CollisionLayerNames::Walls]),
                CollisionLayer::new(vec![]),
                CollisionLayer::new(vec![]),
            ],
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct CollideEvent {
    pub a: Entity,
    pub b: Entity,
}

impl CollideEvent {
    fn new(a: Entity, b: Entity) -> Self {
        Self { a: a, b: b }
    }
}

#[derive(Event)]
pub struct UniqueCollideEvent {
    pub a: Entity,
    pub b: Entity,
}

impl UniqueCollideEvent {
    fn new(collide_event: CollideEvent) -> Self {
        Self {
            a: collide_event.a,
            b: collide_event.b,
        }
    }
}

#[derive(Resource, Default)]
pub struct CollideEventsThisFrame(pub Vec<(Entity, Entity)>);

pub fn find_collisions(
    circle_collider_query: Query<(&CircleCollider, &Transform)>,
    aa_rect_collider_query: Query<(&AARectCollider, &Transform)>,
    collision_layers: Res<CollisionLayers>,
    quad_tree: Res<QuadTree>,
    mut collide_event_writer: EventWriter<CollideEvent>,
    mut unique_collide_event_writer: EventWriter<UniqueCollideEvent>,
    mut collide_events_this_frame: ResMut<CollideEventsThisFrame>,
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
                    &aa_rect_collider_query,
                    &quad_tree,
                    &mut collide_event_writer,
                    &mut unique_collide_event_writer,
                    &layer.collides_with,
                    &mut collide_events_this_frame,
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
    aa_rect_collider_query: &Query<(&AARectCollider, &Transform)>,
    quad_tree: &Res<QuadTree>,
    collide_event_writer: &mut EventWriter<CollideEvent>,
    unique_collide_event_writer: &mut EventWriter<UniqueCollideEvent>,
    collides_with: &Vec<CollisionLayerNames>,
    collide_events_this_frame: &mut ResMut<CollideEventsThisFrame>,
) {
    for b_entity in quad_tree.query_range(&AABB::new(
        a_translation,
        a.radius + COLLIDER_CHECK_DISTANCE,
    )) {
        if let Ok((b, b_transform)) = circle_collider_query.get(b_entity) {
            if collides_with.contains(&b.layer) {
                if circle_circle_collision(a, a_translation, b, b_transform.translation.xy()) {
                    let collide_event = CollideEvent::new(*a_entity, b_entity);
                    collide_event_writer.send(collide_event);
                    if !collide_events_this_frame.0.contains(&(*a_entity, b_entity)) {
                        unique_collide_event_writer.send(UniqueCollideEvent::new(collide_event));
                        collide_events_this_frame.0.push((*a_entity, b_entity));
                    }
                }
            }
        } else if let Ok((b, b_transform)) = aa_rect_collider_query.get(b_entity) {
            if collides_with.contains(&b.layer) {
                if circle_aa_rect_collision(a, a_translation, b, b_transform.translation.xy()) {
                    let collide_event = CollideEvent::new(*a_entity, b_entity);
                    collide_event_writer.send(collide_event);
                    if !collide_events_this_frame.0.contains(&(*a_entity, b_entity)) {
                        unique_collide_event_writer.send(UniqueCollideEvent::new(collide_event));
                        collide_events_this_frame.0.push((*a_entity, b_entity));
                    }
                }
            }
        }
    }
}

pub fn handle_collisions(
    //only works when a is circle collider and b is rect collider
    //undefined behaviour when two "use_collisions" entities collide
    mut collide_event_reader: EventReader<CollideEvent>,
    mut circle_query: Query<
        (&Physics, &mut Transform, &CircleCollider, &mut Velocity),
        Without<AARectCollider>,
    >,
    rect_query: Query<(&Transform, &AARectCollider), Without<CircleCollider>>,
) {
    for event in collide_event_reader.read() {
        if let Ok((physics_a, mut transform_a, collider_a, mut velocity_a)) =
            circle_query.get_mut(event.a)
        {
            if physics_a.use_collisions {
                if let Ok((transform_b, collider_b)) = rect_query.get(event.b) {
                    let p = transform_a.translation.xy();
                    let (contact_point, in_rect) =
                        collider_b.nearest_point(p, transform_b.translation.xy());
                    let mut normal = p - contact_point;
                    let mut distance = normal.length();
                    normal = normal.normalize();
                    if in_rect {
                        normal *= -1.;
                        distance *= -1.;
                    }

                    let offset = normal * (collider_a.radius - distance);
                    transform_a.translation += Vec3::new(offset.x, offset.y, 0.);

                    let dot_product = normal.dot(velocity_a.0);
                    velocity_a.0 -= normal * dot_product;
                }
            }
        }
    }
}
