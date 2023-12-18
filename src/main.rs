#![feature(variant_count)]

pub mod game;

use bevy::{
    prelude::*,
    window::PresentMode,
};
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {primary_window: Some (Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }), ..default()}),
            GamePlugin,
        ))
        .add_systems(PreStartup, spawn_camera)
        .run();
}

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(
    mut commands: Commands
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}
