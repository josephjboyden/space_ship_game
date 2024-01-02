use bevy::{math, prelude::*};
use noise::{NoiseFn, OpenSimplex};

use super::PLAYER_AREA_HALF_DIMENTION;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate);
    }
}

const N: usize = 200;
const M: usize = N;
const ZOOM: f64 = 0.5;
const THREASHOLD: f32 = 0.;

const TILE_WIDTH: f32 = PLAYER_AREA_HALF_DIMENTION * 2.0 / N as f32;
const TILE_HEIGHT: f32 = PLAYER_AREA_HALF_DIMENTION * 2.0 / M as f32;

const TWO_PI: f64 = std::f64::consts::PI * 2.;

fn generate(mut commands: Commands) {
    let noise = OpenSimplex::new(0);

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

            if value > THREASHOLD {
                commands.spawn(SpriteBundle {
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
                });
            }
        }
    }
}
