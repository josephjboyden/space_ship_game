mod gunner;
mod pilot;

use std::ops::Deref;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use super::{
    aliens::Alien,
    health::HealthRunoutEvent,
    health::{ChangeHealthEvent, ChangeHealthMode, Health},
    physics::{
        Acceleration, CircleCollider, CollideEvent, CollisionLayerNames, CollisionLayers, Physics,
        Velocity,
    },
    GameOverEvent,
};

use crate::MainCamera;

use gunner::GunnerPlugin;
use pilot::PilotPlugin;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PilotPlugin, GunnerPlugin))
            .add_systems(Startup, spawn_ship)
            .add_systems(Update, (check_collisions, check_runout))
            .add_systems(PostUpdate, (move_camera,));
    }
}

#[derive(Component)]
pub struct Ship {
    pub target_direction: Vec2,
}

impl Default for Ship {
    fn default() -> Self {
        Self {
            target_direction: Vec2 { x: 0., y: 1. },
        }
    }
}

#[derive(Component)]
pub struct Gun {
    direction: Vec2,
    projectile_spawn: f32,
    last_fired: f32,
}
impl Gun {
    fn new(projectile_spawn: f32) -> Self {
        Self {
            direction: Vec2::ZERO,
            projectile_spawn: projectile_spawn,
            last_fired: 0.,
        }
    }
}

const SHIP_SIZE: f32 = 60. / 128.;

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut collision_layers: ResMut<CollisionLayers>,
) {
    let ship_entity = commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("ship.png"),
                transform: Transform {
                    translation: Vec3::new(0., 0., 1.),
                    scale: Vec3::new(SHIP_SIZE, SHIP_SIZE, 1.),
                    ..default()
                },
                ..default()
            },
            Physics::default(),
            Acceleration {
                local: true,
                ..default()
            },
            Velocity::default(),
            Ship::default(),
            Health::new(100.),
            CircleCollider::new(30.),
        ))
        .with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(shape::Quad::new(Vec2::new(20., 150.)).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::GRAY)),
                    transform: Transform::from_translation(Vec3::new(0., 75., 0.9)),
                    ..default()
                },
                Gun::new(75. * SHIP_SIZE),
            ));
        })
        .id();
    collision_layers.layers[CollisionLayerNames::Ship as usize]
        .in_layer
        .push(ship_entity);
}

fn move_camera(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Ship>)>,
    ship_query: Query<&Transform, (With<Ship>, Without<MainCamera>)>,
) {
    if let (Ok(mut camera_transform), Ok(ship_transform)) =
        (camera_query.get_single_mut(), ship_query.get_single())
    {
        camera_transform.translation = ship_transform.translation.clone();
    }
}

fn check_collisions(
    mut commands: Commands,
    ship_query: Query<Entity, (With<Ship>, Without<Alien>)>,
    alien_query: Query<Entity, (With<Alien>, Without<Ship>)>,
    mut change_health_event_writer: EventWriter<ChangeHealthEvent>,
    mut collide_event_reader: EventReader<CollideEvent>,
) {
    if let Ok(ship) = ship_query.get_single() {
        for event in collide_event_reader.read() {
            if event.a == ship {
                if let Ok(alien) = alien_query.get(event.b) {
                    change_health_event_writer.send(ChangeHealthEvent::new(
                        0.,
                        ChangeHealthMode::Damage,
                        ship,
                    ));
                    commands.entity(alien).despawn();
                }
            }
        }
    }
}

fn check_runout(
    mut commands: Commands,
    mut game_over_event_writer: EventWriter<GameOverEvent>,
    mut health_runout_event_reader: EventReader<HealthRunoutEvent>,
    ship_query: Query<Entity, With<Ship>>,
) {
    if let Ok(ship_entity) = ship_query.get_single() {
        for event in health_runout_event_reader.read() {
            if event.0 == ship_entity {
                commands.entity(ship_entity).despawn_recursive();
                game_over_event_writer.send(GameOverEvent);
            }
        }
    }
}
