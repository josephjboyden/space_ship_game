use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use rand::prelude::*;

use super::{quad_tree::QuadTreeElement, PLAYER_AREA_HALF_DIMENTION};

pub struct AsteroidsPlugin;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_asteroids);
    }
}

#[derive(Component)]
pub struct Asteroid {
    pub radius: f32,
}

impl Asteroid {
    fn new(radius: f32) -> Asteroid {
        Asteroid { radius: radius }
    }
}

const SPAWN_RANGE: f32 = PLAYER_AREA_HALF_DIMENTION;

fn spawn_asteroids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..800 {
        let radius: f32 = rng.gen_range(10.0..50.0);
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::ORANGE_RED)),
                transform: Transform::from_translation(Vec3::new(
                    rng.gen_range(-SPAWN_RANGE..SPAWN_RANGE),
                    rng.gen_range(-SPAWN_RANGE..SPAWN_RANGE),
                    0.1,
                )),
                ..default()
            },
            Asteroid::new(radius),
            QuadTreeElement,
        ));
    }
}
