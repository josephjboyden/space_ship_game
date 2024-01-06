use bevy::prelude::*;
use noise::{NoiseFn, OpenSimplex};

use super::{
    aliens::alien_avoid::AARectAlienAvoid,
    physics::{AARectCollider, CollisionLayerNames, CollisionLayers},
    quad_tree::QuadTreeElement,
    PLAYER_AREA_HALF_DIMENTION,
};

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, show_world)
            .insert_resource(World::generate());
    }
}

const N: usize = (PLAYER_AREA_HALF_DIMENTION * 2.0 / 50.0) as usize;
const M: usize = N;
const ZOOM: f64 = 5000. / PLAYER_AREA_HALF_DIMENTION as f64;
const THREASHOLD_A: f32 = 0.;
const THREASHOLD_B: f32 = 0.3;

const TILE_WIDTH: f32 = PLAYER_AREA_HALF_DIMENTION * 2.0 / N as f32;
const TILE_HEIGHT: f32 = PLAYER_AREA_HALF_DIMENTION * 2.0 / M as f32;

const TWO_PI: f64 = std::f64::consts::PI * 2.;

#[derive(Resource)]
pub struct World {
    pub world_data: [[bool; M]; N],
}

impl World {
    fn generate() -> Self {
        let noise = OpenSimplex::new(7);
        let mut world_data: [[bool; M]; N] = [[false; M]; N];

        for j in 0..M {
            for i in 0..N {
                //needs to be tilable: samples the surface of a 2-torus in R4
                //scale to between 0 and 1;
                let (s, t) = (i as f64 / N as f64, j as f64 / M as f64);

                // Calculate 4D coordinates
                let x = (s * TWO_PI).cos() / ZOOM;
                let y = (t * TWO_PI).cos() / ZOOM;
                let z = (s * TWO_PI).sin() / ZOOM;
                let w = (t * TWO_PI).sin() / ZOOM;

                let value = noise.get([x, y, z, w]) as f32; // -1 < value < 1

                world_data[i][j] = value > THREASHOLD_A && value < THREASHOLD_B;
            }
        }

        Self {
            world_data: world_data,
        }
    }
}

fn show_world(
    mut commands: Commands,
    world: Res<World>,
    mut collision_layers: ResMut<CollisionLayers>,
) {
    for j in 0..M {
        for i in 0..N {
            if !world.world_data[i][j] {
                let wall_entity = commands
                    .spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(1., 1., 1.),
                                custom_size: Some(Vec2::new(TILE_WIDTH, TILE_HEIGHT)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(
                                TILE_WIDTH * i as f32,
                                TILE_HEIGHT * j as f32,
                                0.,
                            )),
                            ..default()
                        },
                        AARectAlienAvoid {
                            half_size: Vec2::new(TILE_WIDTH, TILE_HEIGHT) / 2.,
                        },
                        QuadTreeElement,
                        AARectCollider::new(
                            Vec2::new(TILE_WIDTH, TILE_HEIGHT),
                            CollisionLayerNames::Walls,
                        ),
                    ))
                    .id();
                collision_layers.layers[CollisionLayerNames::Walls as usize]
                    .in_layer
                    .push(wall_entity);
            }
        }
    }
}

pub fn world_to_grid(x: f32, y: f32) -> (usize, usize) {
    ((x / TILE_WIDTH) as usize, (y / TILE_HEIGHT) as usize)
}
