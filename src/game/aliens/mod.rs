pub mod alien_avoid;

use bevy::prelude::*;

use num::clamp;
use rand::prelude::*;
use std::collections::HashMap;

use alien_avoid::AARectAlienAvoid;

use super::{
    health::{Health, HealthRunoutEvent},
    health_pack::SpawnHelthPackEvent,
    physics::{CircleCollider, CollisionLayerNames, CollisionLayers, Physics, Velocity},
    quad_tree::*,
    score::Score,
    ship::Ship,
    world_generation::{world_to_grid, World},
    PLAYER_AREA_HALF_DIMENTION,
};

pub struct AliensPlugin;

impl Plugin for AliensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuadTreePlugin)
            .add_systems(Startup, spawn_aliens)
            .add_systems(Update, (simulate_boids, check_bounds, check_runout))
            .add_systems(PostUpdate, point_to_velocity);
    }
}

#[derive(Component, PartialEq, Eq, Hash)]
pub struct Alien;

impl Default for Alien {
    fn default() -> Self {
        Alien {}
    }
}

const SPAWN_RANGE: f32 = PLAYER_AREA_HALF_DIMENTION * 2.;
const SPAWN_DENSTIY: f32 = 0.00002;
const NUM: u32 = (SPAWN_RANGE * SPAWN_RANGE * SPAWN_DENSTIY) as u32;
const SPEED: f32 = 100.0;
pub const ALIEN_RADIUS: f32 = 15.;
const ALIEN_SIZE: f32 = ALIEN_RADIUS * 2. / 64.;
const HEALTH: f32 = 1.0;

fn spawn_aliens(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_layers: ResMut<CollisionLayers>,
    world: Res<World>,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..NUM {
        let forward = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize();

        let x: f32 = rng.gen_range(0.0..SPAWN_RANGE);
        let y: f32 = rng.gen_range(0.0..SPAWN_RANGE);

        let (i, j) = world_to_grid(x, y);

        if !world.world_data[i][j] {
            continue;
        }

        let alien_entity = commands
            .spawn((
                SpriteBundle {
                    texture: asset_server.load("alien.png"),
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.2),
                        scale: Vec3::new(ALIEN_SIZE, ALIEN_SIZE, 1.),
                        rotation: Quat::from_rotation_z(Vec2::X.angle_between(forward)),
                    },
                    ..default()
                },
                Physics::default(),
                Velocity(forward * SPEED),
                Alien::default(),
                CircleCollider::new(15., CollisionLayerNames::Aliens),
                Health::new(HEALTH),
                QuadTreeElement,
            ))
            .id();
        collision_layers.layers[CollisionLayerNames::Aliens as usize]
            .in_layer
            .push(alien_entity);
    }
}

const RADIUS: f32 = 200.0;
const VISION_CONE_THRESHOLD: f32 = -0.7;
const SEPERATION_RADIUS: f32 = 70.;
const ROTATION_SPEED: f32 = 8.;
const ALIEN_AVOID_SEPERATION_RADIUS: f32 = 100.;
const SHIP_SEARCH_RADIUS: f32 = 200.;

const SEPERATION: f32 = 10.;
const ALINGMENT: f32 = 3.;
const COHESION: f32 = 1.;
const SHIP_SEARCH: f32 = 5.;
const AARECT_AVOIDANCE: f32 = 50.;

fn in_view(forward: Vec2, direction: Vec2) -> bool {
    direction.normalize().dot(forward.normalize()) > VISION_CONE_THRESHOLD
}

fn per_boid_calcs(
    current: &mut (Vec2, Vec2, Vec2, Vec2),
    direction: Vec2,
    distance: f32,
    velocity: Vec2,
) {
    let seperation = (1. - clamp(distance / SEPERATION_RADIUS, 0., 1.)) * direction.normalize();
    let alingment = velocity;
    let cohesion = direction;
    current.0 += seperation;
    current.1 += alingment;
    current.2 += cohesion;
}

fn per_alien_avoid_calcs(current: &mut (Vec2, Vec2, Vec2, Vec2), direction: Vec2, distance: f32) {
    current.3 +=
        (1. - clamp(distance / ALIEN_AVOID_SEPERATION_RADIUS, 0., 1.)) * direction.normalize();
}

fn ship_search(
    ship_query: &Query<&Transform, With<Ship>>,
    alien_transform: &Transform,
    alien_forward: Vec2,
) -> Vec2 {
    let mut seperation = Vec2::ZERO;
    if let Ok(ship_transform) = ship_query.get_single() {
        let direction = (ship_transform.translation - alien_transform.translation).xy();
        let distance = direction.length();

        if distance < SHIP_SEARCH_RADIUS && in_view(alien_forward, direction) {
            seperation +=
                (1. / clamp(distance / (SHIP_SEARCH_RADIUS), 0., 1.)) * direction.normalize();
        }
    }
    return seperation;
}

fn turn_towards(to_target: Vec2, forward: &mut Vec2, angle: f32) {
    if to_target.length() == 0.0 {
        return;
    };

    let tt3 = Vec3::new(to_target.x, to_target.y, 0.).normalize();
    let mut f3 = Vec3::new(forward.x, forward.y, 0.);
    let z = tt3.cross(f3.normalize()).z;

    let mut angular_velocity = 0.;

    if z > 0.0001 {
        angular_velocity = -angle;
    } else if z < -0.0001 {
        angular_velocity = angle;
    }

    if angular_velocity != 0. {
        *forward = Quat::from_axis_angle(Vec3::Z, angular_velocity)
            .mul_vec3(f3)
            .xy();

        f3 = Vec3::new(forward.x, forward.y, 0.);

        if z * tt3.cross(f3.normalize()).z < 0. {
            *forward = to_target.normalize() * forward.length();
        }
    }
}

fn boid_task(
    quad_tree: &Res<QuadTree>,
    alien_query: &Query<(Entity, &Transform, &mut Velocity), With<Alien>>,
    alien_avoid_query: &Query<(&Transform, &AARectAlienAvoid)>,
    transform_1: &Transform,
    velocity_1: &Velocity,
    alien_1: &Entity,
    near_aliens_map: &mut HashMap<u32, (Vec2, Vec2, Vec2, Vec2)>,
    commands: &mut Commands,
) {
    for entity in quad_tree.query_range(&AABB::new(transform_1.translation.xy(), RADIUS)) {
        if let Ok((_, transform_2, velocity_2)) = alien_query.get(entity) {
            let direction = (transform_2.translation - transform_1.translation).xy();
            let distance = direction.length();
            if distance <= RADIUS {
                if in_view(velocity_1.0.xy(), direction) {
                    let near_aliens = near_aliens_map.get_mut(&alien_1.index());

                    match near_aliens {
                        Some(near_aliens) => {
                            per_boid_calcs(near_aliens, direction, distance, velocity_2.0);
                        }
                        None => {
                            let current = &mut (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, Vec2::ZERO);
                            per_boid_calcs(current, direction, distance, velocity_2.0);
                            near_aliens_map.insert(alien_1.index(), *current);
                        }
                    }
                }
            }
        } else if let Ok((rect_transform, rect_alien_avoid)) = alien_avoid_query.get(entity) {
            let direction = (rect_transform.translation - transform_1.translation).xy();
            let distance =
                rect_alien_avoid.dist(transform_1.translation, rect_transform.translation);
            if distance < 0. {
                commands.entity(*alien_1).despawn();
            } else if distance <= ALIEN_AVOID_SEPERATION_RADIUS {
                if in_view(velocity_1.0.xy(), direction) {
                    let near_aliens = near_aliens_map.get_mut(&alien_1.index());

                    match near_aliens {
                        Some(near_aliens) => {
                            per_alien_avoid_calcs(near_aliens, direction, distance);
                        }
                        None => {
                            let current = &mut (Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, Vec2::ZERO);
                            per_alien_avoid_calcs(current, direction, distance);
                            near_aliens_map.insert(alien_1.index(), *current);
                        }
                    }
                }
            }
        }
    }
}

fn simulate_boids(
    mut alien_query: Query<(Entity, &Transform, &mut Velocity), With<Alien>>,
    alien_avoid_query: Query<(&Transform, &AARectAlienAvoid)>,
    ship_query: Query<&Transform, With<Ship>>,
    time: Res<Time>,
    quad_tree: Res<QuadTree>,
    mut commands: Commands,
) {
    let mut near_aliens_map: HashMap<u32, (Vec2, Vec2, Vec2, Vec2)> = HashMap::new();

    //let mut iter = alien_query.iter_combinations_mut();
    //while let Some([(alien_1, transform_1, velocity_1), (alien_2, transform_2, velocity_2)]) = iter.fetch_next() {
    for (alien_1, transform_1, velocity_1) in alien_query.iter() {
        boid_task(
            &quad_tree,
            &alien_query,
            &alien_avoid_query,
            &transform_1,
            &velocity_1,
            &alien_1,
            &mut near_aliens_map,
            &mut commands,
        )
    }

    for (alien_entity, alien_transform, mut alien_velocity) in alien_query.iter_mut() {
        let mut turn_target = Vec2::ZERO;

        let near_aliens = near_aliens_map.get(&alien_entity.index());
        match near_aliens {
            Some(near_aliens) => {
                turn_target = -SEPERATION * (near_aliens.0.normalize_or_zero())
                    + ALINGMENT * (near_aliens.1.normalize_or_zero())
                    + COHESION * (near_aliens.2.normalize_or_zero())
                    + -AARECT_AVOIDANCE * near_aliens.3
            }
            None => {}
        }

        turn_target += SHIP_SEARCH
            * ship_search(&ship_query, alien_transform, alien_velocity.0.clone())
                .normalize_or_zero();

        turn_towards(
            turn_target,
            &mut alien_velocity.0,
            time.delta_seconds() * ROTATION_SPEED,
        );
    }
}

const BOUND: f32 = SPAWN_RANGE;

fn check_bounds(mut aliens_query: Query<&mut Transform, With<Alien>>) {
    for mut alien_transform in aliens_query.iter_mut() {
        if alien_transform.translation.x > BOUND {
            alien_transform.translation.x = 0.
        }
        if alien_transform.translation.x < 0. {
            alien_transform.translation.x = BOUND
        }
        if alien_transform.translation.y > BOUND {
            alien_transform.translation.y = 0.
        }
        if alien_transform.translation.y < 0. {
            alien_transform.translation.y = BOUND
        }
    }
}

fn point_to_velocity(mut alien_query: Query<(&Velocity, &mut Transform), With<Alien>>) {
    for (alien_velocity, mut alien_transform) in alien_query.iter_mut() {
        alien_transform.rotation = Quat::from_rotation_z(Vec2::X.angle_between(alien_velocity.0));
    }
}

fn check_runout(
    mut commands: Commands,
    mut health_runout_event_reader: EventReader<HealthRunoutEvent>,
    alien_query: Query<(Entity, &Transform), With<Alien>>,
    mut score: ResMut<Score>,
    mut spawn_health_pack_event_writer: EventWriter<SpawnHelthPackEvent>,
) {
    for event in health_runout_event_reader.read() {
        if let Ok((alien_entity, alien_transform)) = alien_query.get(event.0) {
            commands.entity(alien_entity).despawn();
            score.0 += 1;
            spawn_health_pack_event_writer
                .send(SpawnHelthPackEvent::new(alien_transform.translation.xy()))
        }
    }
}
