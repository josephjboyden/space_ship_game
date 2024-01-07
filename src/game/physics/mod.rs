pub mod collision;
pub mod components;
pub mod movement;

use bevy::prelude::*;

use movement::AddImpulseEvent;

use collision::{
    find_collisions, handle_collisions, CollideEvent, CollideEventsThisFrame, CollisionLayers,
    UniqueCollideEvent,
};
use movement::{acceleration_physics_update, apply_impulse, velocity_physics_update};

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
