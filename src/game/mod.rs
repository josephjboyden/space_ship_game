pub mod physics;
pub mod ship;
mod asteroids;
mod aliens;
mod player;
mod hud;
pub mod health;
pub mod score;
mod quad_tree;

use bevy::prelude::*;


use health::HealthPlugin;
use score::Score;
use ship::ShipPlugin;
use physics::PhysicsPlugin;
use asteroids::AsteroidsPlugin;
use aliens::AliensPlugin;
use player::PlayerPlugin;
use hud::HUDPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(& self, app: &mut App) {
        app .add_event::<GameOverEvent>()
            .insert_resource(Score(0))
            .add_plugins((
                PhysicsPlugin,
                ShipPlugin,
                AsteroidsPlugin,
                AliensPlugin,
                PlayerPlugin,
                HUDPlugin,
                HealthPlugin,
            ));
    }
}

#[derive(Event)]
pub struct GameOverEvent;

pub const PLAYER_AREA_HALF_DIMENTION: f32 = 5000.;
