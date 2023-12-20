mod aliens;
mod asteroids;
pub mod health;
mod hud;
pub mod physics;
mod player;
mod quad_tree;
pub mod score;
pub mod ship;

use bevy::prelude::*;

use aliens::AliensPlugin;
use asteroids::AsteroidsPlugin;
use health::HealthPlugin;
use hud::HUDPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use score::Score;
use ship::ShipPlugin;

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
            ));
    }
}

#[derive(Event)]
pub struct GameOverEvent;

pub const PLAYER_AREA_HALF_DIMENTION: f32 = 5000.;
