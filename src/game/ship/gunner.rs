use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use super::super::{
    aliens::Alien,
    health::{ChangeHealthEvent, ChangeHealthMode, HealthSet},
    physics::{
        AddImpulseEvent, CircleCollider, CollisionLayerNames, CollisionLayers, Mass, Physics,
        UniqueCollideEvent, Velocity,
    },
    player::{PlayerManager, Role},
};
use super::{Gun, Ship};

pub struct GunnerPlugin;

impl Plugin for GunnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireEvent>()
            .add_systems(
                PreUpdate,
                (
                    handle_mouse_movement,
                    handle_mouse_buttons,
                    handle_gamepad_input,
                    handle_stick_input,
                ),
            )
            .add_systems(
                Update,
                (
                    gun_fired,
                    despawn_projectiles,
                    check_projectile_collisions.in_set(HealthSet::Write),
                ),
            );
    }
}

fn handle_mouse_movement(
    window_query: Query<&Window>,
    mut gun_query: Query<(&mut Transform, &mut Gun), Without<Ship>>,
    ship_query: Query<&Transform, (With<Ship>, Without<Gun>)>,
    player_manager: ResMut<PlayerManager>,
) {
    match player_manager.get_input_scheme(Role::Gunner) {
        Some(input_scheme) => {
            if !input_scheme.is_keyboard() {
                return;
            };
        }
        None => return,
    }

    if let Ok((mut gun_transform, mut gun)) = gun_query.get_single_mut() {
        if let Ok(ship_transform) = ship_query.get_single() {
            let window = window_query.single();

            if let Some(mut mouse_position) = window.cursor_position().clone() {
                mouse_position -= Vec2 {
                    x: window.width() / 2.,
                    y: window.height() / 2.,
                };
                mouse_position.y *= -1.;

                gun.direction = mouse_position.clone().normalize();

                let rotation = Quat::from_rotation_z(
                    Vec2::Y.angle_between(mouse_position)
                        - Vec2::Y.angle_between(ship_transform.rotation.mul_vec3(Vec3::Y).xy()),
                );

                gun_transform.translation =
                    rotation.mul_vec3(Vec3::Y * gun_transform.translation.length());
                gun_transform.rotation = rotation;
            }
        }
    }
}

fn handle_stick_input(
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    mut gun_query: Query<(&mut Transform, &mut Gun), Without<Ship>>,
    ship_query: Query<&Transform, (With<Ship>, Without<Gun>)>,
    player_manager: ResMut<PlayerManager>,
) {
    match player_manager.get_input_scheme(Role::Gunner) {
        Some(input_scheme) => {
            if !input_scheme.is_controller() {
                return;
            };
        }
        None => return,
    }

    if let Ok((mut gun_transform, mut gun)) = gun_query.get_single_mut() {
        if let Ok(ship_transform) = ship_query.get_single() {
            for gamepad in gamepads.iter() {
                let left_stick = Vec2::new(
                    axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
                        .unwrap(),
                    axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
                        .unwrap(),
                );
                if left_stick.length() != 0. {
                    gun.direction = left_stick.normalize();

                    let rotation = Quat::from_rotation_z(
                        Vec2::Y.angle_between(gun.direction)
                            - Vec2::Y.angle_between(ship_transform.rotation.mul_vec3(Vec3::Y).xy()),
                    );

                    gun_transform.translation =
                        rotation.mul_vec3(Vec3::Y * gun_transform.translation.length());
                    gun_transform.rotation = rotation;
                }
            }
        }
    }
}

#[derive(Event, Debug)]
struct FireEvent {
    position: Vec2,
    direction: Vec2,
    velocity: Velocity,
}

impl FireEvent {
    fn new(position: Vec2, direction: Vec2, velocity: Velocity) -> Self {
        Self {
            position: position,
            direction: direction,
            velocity: velocity,
        }
    }
}

const FIRE_RATE: f32 = 10.;
const FIRE_INTERVAL: f32 = 1. / FIRE_RATE;

fn handle_mouse_buttons(
    buttons: Res<Input<MouseButton>>,
    player_manager: ResMut<PlayerManager>,
    mut fire_event_writer: EventWriter<FireEvent>,
    gun_query: Query<(&GlobalTransform, &Gun)>,
    ship_query: Query<&Velocity, With<Ship>>,
    time: Res<Time>,
) {
    match player_manager.get_input_scheme(Role::Gunner) {
        Some(input_scheme) => {
            if !input_scheme.is_keyboard() {
                return;
            };
        }
        None => return,
    }

    if let Ok((gun_transform, gun)) = gun_query.get_single() {
        if let Ok(ship_velocity) = ship_query.get_single() {
            if buttons.pressed(MouseButton::Left)
                && time.elapsed_seconds_wrapped() - gun.last_fired > FIRE_INTERVAL
            {
                fire_event_writer.send(FireEvent::new(
                    gun_transform.translation().xy() + gun.direction * gun.projectile_spawn,
                    gun.direction,
                    ship_velocity.clone(),
                ))
            }
        }
    }
}

fn handle_gamepad_input(
    gamepads: Res<Gamepads>,
    button: Res<Input<GamepadButton>>,
    player_manager: ResMut<PlayerManager>,
    mut fire_event_writer: EventWriter<FireEvent>,
    gun_query: Query<(&GlobalTransform, &Gun)>,
    ship_query: Query<&Velocity, With<Ship>>,
    time: Res<Time>,
) {
    match player_manager.get_input_scheme(Role::Gunner) {
        Some(input_scheme) => {
            if !input_scheme.is_controller() {
                return;
            };
        }
        None => return,
    }

    if let Ok((gun_transform, gun)) = gun_query.get_single() {
        if let Ok(ship_velocity) = ship_query.get_single() {
            for gamepad in gamepads.iter() {
                let right_trigger = GamepadButton::new(gamepad, GamepadButtonType::RightTrigger2);
                if button.pressed(right_trigger)
                    && time.elapsed_seconds_wrapped() - gun.last_fired > FIRE_INTERVAL
                {
                    fire_event_writer.send(FireEvent::new(
                        gun_transform.translation().xy() + gun.direction * gun.projectile_spawn,
                        gun.direction,
                        ship_velocity.clone(),
                    ))
                }
            }
        }
    }
}

#[derive(Component)]
struct Projectile {
    time_of_creation: f32,
}

const PROJECTILE_SPEED: f32 = 200.;

fn gun_fired(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fire_event_reader: EventReader<FireEvent>,
    time: Res<Time>,
    mut gun_query: Query<&mut Gun>,
    mut collision_layers: ResMut<CollisionLayers>,
    ship_query: Query<Entity, With<Ship>>,
    mut add_impulse_event_writer: EventWriter<AddImpulseEvent>,
) {
    for event in fire_event_reader.read() {
        if let Ok(mut gun) = gun_query.get_single_mut() {
            gun.last_fired = time.elapsed_seconds_wrapped()
        }
        let velocity = event.direction * PROJECTILE_SPEED + event.velocity.0;
        let mass: f32 = 1_000.0;
        let projectile_entity = commands
            .spawn((
                Projectile {
                    time_of_creation: time.elapsed_seconds_wrapped(),
                },
                Velocity(velocity),
                Physics::default(),
                Mass(mass),
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(10.).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::ORANGE_RED)),
                    transform: Transform::from_translation(Vec3::new(
                        event.position.x,
                        event.position.y,
                        0.3,
                    )),
                    ..default()
                },
                CircleCollider::new(10., CollisionLayerNames::CollidesWithAliens),
            ))
            .id();
        collision_layers.layers[CollisionLayerNames::CollidesWithAliens as usize]
            .in_layer
            .push(projectile_entity);
        if let Ok(ship) = ship_query.get_single() {
            add_impulse_event_writer.send(AddImpulseEvent::new(-velocity, mass, ship));
        }
    }
}

const PROJECTILE_LIFETIME: f32 = 2.;

fn despawn_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile)>,
    time: Res<Time>,
) {
    for (projectile_entity, projectile) in projectile_query.iter() {
        if time.elapsed_seconds_wrapped() - projectile.time_of_creation > PROJECTILE_LIFETIME {
            commands.entity(projectile_entity).despawn();
        }
    }
}

fn check_projectile_collisions(
    mut commands: Commands,
    projectile_query: Query<Entity, (With<Projectile>, Without<Alien>)>,
    alien_query: Query<Entity, (With<Alien>, Without<Projectile>)>,
    mut change_health_event_writer: EventWriter<ChangeHealthEvent>,
    mut unique_collide_event_reader: EventReader<UniqueCollideEvent>,
) {
    for event in unique_collide_event_reader.read() {
        if let Ok(projectile) = projectile_query.get(event.a) {
            if let Ok(alien) = alien_query.get(event.b) {
                commands.entity(projectile).despawn();
                change_health_event_writer.send(ChangeHealthEvent::new(
                    10.,
                    ChangeHealthMode::Damage,
                    alien,
                ));
            }
        }
    }
}
