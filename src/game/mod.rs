mod aliens;
mod asteroids;
pub mod health;
pub mod health_pack;
mod hud;
pub mod physics;
mod player;
mod quad_tree;
pub mod score;
pub mod ship;

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use aliens::AliensPlugin;
use asteroids::AsteroidsPlugin;
use health::HealthPlugin;
use health_pack::HealthPackPlugin;
use hud::HUDPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use score::Score;
use ship::ShipPlugin;

use crate::MainCamera;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameOverEvent>()
            .insert_resource(Score(0))
            .add_plugins((
                PhysicsPlugin,
                ShipPlugin,
                AsteroidsPlugin,
                AliensPlugin,
                PlayerPlugin,
                HUDPlugin,
                HealthPlugin,
                HealthPackPlugin,
            ))
            .add_systems(Update, loop_camera);
    }
}

#[derive(Event)]
pub struct GameOverEvent;

pub const PLAYER_AREA_HALF_DIMENTION: f32 = 2000.;

#[derive(Component)]
struct SecondaryCamera;

#[derive(Component)]
struct TertiaryCamera;

fn spawn_tertiary_camera_1(commands: &mut Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 2,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        UiCameraConfig { show_ui: false },
        TertiaryCamera,
    ));
}
fn spawn_tertiary_camera_2(commands: &mut Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 3,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        UiCameraConfig { show_ui: false },
        TertiaryCamera,
    ));
}

fn loop_camera(
    mut commands: Commands,
    mut main_camera_query: Query<
        &Transform,
        (
            With<MainCamera>,
            Without<SecondaryCamera>,
            Without<TertiaryCamera>,
        ),
    >,
    mut secondary_camera_query: Query<
        (Entity, &mut Transform),
        (With<SecondaryCamera>, Without<TertiaryCamera>),
    >,
    window_query: Query<&Window>,
    mut tertiary_camera_query: Query<(Entity, &mut Transform), With<TertiaryCamera>>,
    mut gizmos: Gizmos,
) {
    let window = window_query.single();

    if let Ok(main_camera_transform) = main_camera_query.get_single_mut() {
        let half_width = window.resolution.width() / 2.;
        let half_height = window.resolution.height() / 2.;
        gizmos.rect_2d(
            Vec2::new(PLAYER_AREA_HALF_DIMENTION, PLAYER_AREA_HALF_DIMENTION),
            0.,
            Vec2::new(
                PLAYER_AREA_HALF_DIMENTION * 2.,
                PLAYER_AREA_HALF_DIMENTION * 2.,
            ),
            Color::YELLOW,
        );
        if let Ok((secondary_camera_entity, mut secondary_camera_transform)) =
            secondary_camera_query.get_single_mut()
        {
            let mut past_direction = (false, false, false, false);
            if main_camera_transform.translation.x > PLAYER_AREA_HALF_DIMENTION * 2. - half_width {
                past_direction.0 = true;
            } else if main_camera_transform.translation.x < half_width {
                past_direction.1 = true;
            }
            if main_camera_transform.translation.y > PLAYER_AREA_HALF_DIMENTION * 2. - half_height {
                past_direction.2 = true;
            } else if main_camera_transform.translation.y < half_height {
                past_direction.3 = true;
            }

            if !(past_direction.0 || past_direction.1 || past_direction.2 || past_direction.3) {
                commands.entity(secondary_camera_entity).despawn();
                return;
            }

            secondary_camera_transform.translation = main_camera_transform.translation;

            if past_direction.0 {
                secondary_camera_transform.translation.x -= PLAYER_AREA_HALF_DIMENTION * 2.;
            } else if past_direction.1 {
                secondary_camera_transform.translation.x += PLAYER_AREA_HALF_DIMENTION * 2.;
            }

            if past_direction.2 {
                secondary_camera_transform.translation.y -= PLAYER_AREA_HALF_DIMENTION * 2.;
            } else if past_direction.3 {
                secondary_camera_transform.translation.y += PLAYER_AREA_HALF_DIMENTION * 2.;
            }

            let mut tertiary_cameras: Vec<Option<(Entity, Mut<Transform>)>> = vec![None, None];
            let mut i = 0;
            for mut result in tertiary_camera_query.iter_mut() {
                if i > 1 {
                    println!("to many tertiary cameras");
                    break;
                }
                result.1.translation = main_camera_transform.translation;
                tertiary_cameras[i] = Some(result);
                i += 1;
            }
            if past_direction.0 && past_direction.2 {
                match tertiary_cameras[0].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.x -= PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_1(&mut commands);
                    }
                }
                match tertiary_cameras[1].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.y -= PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_2(&mut commands);
                    }
                }
            } else if past_direction.0 && past_direction.3 {
                match tertiary_cameras[0].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.x -= PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_1(&mut commands);
                    }
                }
                match tertiary_cameras[1].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.y += PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_2(&mut commands);
                    }
                }
            } else if past_direction.1 && past_direction.2 {
                match tertiary_cameras[0].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.x += PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_1(&mut commands);
                    }
                }
                match tertiary_cameras[1].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.y -= PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_2(&mut commands);
                    }
                }
            } else if past_direction.1 && past_direction.3 {
                match tertiary_cameras[0].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.x += PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_1(&mut commands);
                    }
                }
                match tertiary_cameras[1].as_mut() {
                    Some(tertiary_camera) => {
                        tertiary_camera.1.translation.y += PLAYER_AREA_HALF_DIMENTION * 2.
                    }
                    None => {
                        spawn_tertiary_camera_2(&mut commands);
                    }
                }
            } else {
                match tertiary_cameras[0].as_ref() {
                    Some(tertiary_camera) => commands.entity(tertiary_camera.0).despawn(),
                    None => {}
                }
                match tertiary_cameras[1].as_ref() {
                    Some(tertiary_camera) => commands.entity(tertiary_camera.0).despawn(),
                    None => {}
                }
            }
        } else {
            if main_camera_transform.translation.x > PLAYER_AREA_HALF_DIMENTION * 2. - half_width
                || main_camera_transform.translation.x < half_width
                || main_camera_transform.translation.y
                    > PLAYER_AREA_HALF_DIMENTION * 2. - half_height
                || main_camera_transform.translation.y < half_height
            {
                commands.spawn((
                    Camera2dBundle {
                        camera: Camera {
                            order: 1,
                            ..default()
                        },
                        camera_2d: Camera2d {
                            clear_color: ClearColorConfig::None,
                            ..default()
                        },
                        ..default()
                    },
                    UiCameraConfig { show_ui: false },
                    SecondaryCamera,
                ));
            }
        }
    }
}
