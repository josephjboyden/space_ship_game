use bevy::prelude::*;

use super::quad_tree::{QuadTree, AABB};
use num_derive::FromPrimitive;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(0.005))
            .insert_resource(CollideEventsThisFrame(vec![]))
            .add_event::<CollideEvent>()
            .add_event::<UniqueCollideEvent>()
            .add_event::<AddImpulseEvent>()
            .insert_resource(CollisionLayers::default())
            .configure_sets(
                FixedUpdate,
                (
                    PhysicsSet::Changes,
                    PhysicsSet::Movement,
                    PhysicsSet::CollisionDetection,
                    PhysicsSet::CollisionHandling,
                )
                    .chain(),
            )
            //.add_systems(Update, draw_colliders)
            .add_systems(
                FixedUpdate,
                (
                    apply_impulse.in_set(PhysicsSet::Changes),
                    acceleration_physics_update.in_set(PhysicsSet::Movement),
                    velocity_physics_update.in_set(PhysicsSet::Movement),
                    find_collisions.in_set(PhysicsSet::CollisionDetection),
                    handle_collisions.in_set(PhysicsSet::CollisionHandling),
                ),
            );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsSet {
    Changes,
    Movement,
    CollisionDetection,
    CollisionHandling,
}

#[derive(Component)]
pub struct Physics {
    use_collisions: bool,
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

fn acceleration_physics_update(
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

#[derive(Component)]
pub struct AARectCollider {
    pub size: Vec2,
    half_size: Vec2,
    layer: CollisionLayerNames,
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

// fn draw_colliders(
//     mut gizmos: Gizmos,
//     circle_collider_query: Query<(&Transform, &CircleCollider), Without<AARectCollider>>,
//     aa_rect_collider_query: Query<(&Transform, &AARectCollider), Without<CircleCollider>>,
// ) {
//     for (transform, collider) in circle_collider_query.iter() {
//         gizmos.circle_2d(transform.translation.xy(), collider.radius, Color::RED);
//     }
//     for (transform, collider) in aa_rect_collider_query.iter() {
//         gizmos.rect_2d(transform.translation.xy(), 0., collider.size, Color::RED);
//     }
// }

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
struct CollideEventsThisFrame(Vec<(Entity, Entity)>);

fn find_collisions(
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

fn handle_collisions(
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

fn apply_impulse(
    mut add_impulse_event_reader: EventReader<AddImpulseEvent>,
    mut physics_query: Query<(&Mass, &mut Velocity), With<Physics>>,
) {
    for event in add_impulse_event_reader.read() {
        if let Ok((mass, mut velocity)) = physics_query.get_mut(event.entity) {
            velocity.0 += event.value / mass.0
        }
    }
}
