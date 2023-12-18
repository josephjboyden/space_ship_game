use bevy::{
    prelude::*,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
};

use super::{
    score::Score,
    health::Health,
    GameOverEvent,
    ship::Ship
};

pub struct HUDPlugin;

impl Plugin for HUDPlugin {
    fn build(&self, app: &mut App) {
        app .add_systems(Startup, build_hud_root)
            .add_systems(Update, (
                update_fps,
                update_score,
                update_healthbar,
                game_over
            ))
            .add_plugins(FrameTimeDiagnosticsPlugin);
    }
}

fn build_hud_root(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        ..default()
    }).with_children(|parent| {build_hud(parent, asset_server)});
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Healthbar;

#[derive(Component)]
struct GameOverText;

fn build_hud<'a>(parent: &'a mut ChildBuilder, asset_server: Res<AssetServer>) {
    parent.spawn(NodeBundle{
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(25.0),
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            TextBundle::from_sections([
                TextSection {
                    value: "FPS: ".into(),
                    style: TextStyle {
                        font: asset_server.load("fonts/font.ttf"),
                        font_size: 30.0,
                        ..default()
                    }
                },
                TextSection {
                    value: "N/A".into(),
                    style: TextStyle {
                        font: asset_server.load("fonts/font.ttf"),
                        font_size: 30.0,
                        ..default()
                    }
                },
            ]),
            FpsText
        ));

        parent.spawn((
            TextBundle::from_sections([
                TextSection {
                    value: "Score: ".into(),
                    style: TextStyle {
                        font: asset_server.load("fonts/font.ttf"),
                        font_size: 30.0,
                        ..default()
                    }
                },
                TextSection {
                    value: "N/A".into(),
                    style: TextStyle {
                        font: asset_server.load("fonts/font.ttf"),
                        font_size: 30.0,
                        ..default()
                    }
                },
            ]),
            ScoreText
        ));
    });

    parent.spawn(
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(50.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }
    ).with_children(|parent| {
        parent.spawn((
            TextBundle::from_sections([
                TextSection {
                    value: "".into(),
                    style: TextStyle {
                        font: asset_server.load("fonts/font.ttf"),
                        font_size: 80.0,
                        color: Color::RED.into(),
                        ..default()
                    } 
                }
            ]),
            GameOverText,
        ));
    });

    parent.spawn(
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(21.),
                ..default()
            },
            ..default()
        }
    );

    parent.spawn(
        NodeBundle {
            style: Style {
                width: Val::Percent(50.),
                height: Val::Percent(2.),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        }
    ).with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                background_color: Color::RED.into(),
                ..default()
            },
            Healthbar
        ));
    });
}

fn update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        // try to get a "smoothed" FPS value from Bevy
        if let Some(value) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.sections[1].value = format!("{value:>4.0}");
            text.sections[1].style.color = Color::WHITE;
        } else {
            text.sections[1].value = " N/A".into();
            text.sections[1].style.color = Color::GRAY;
        }
    }
}

fn update_score(
    mut query: Query<&mut Text, With<ScoreText>>,
    score: Res<Score>,
) {
    for mut text in &mut query {
        text.sections[1].value = format!("{:?}", score.0)
    }
}

fn update_healthbar(
    mut healthbar_query: Query<&mut Style, With<Healthbar>>,
    ship_query: Query<&Health, With<Ship>>,
) {
    for mut style in  &mut healthbar_query{
        if let Ok(ship_health) = ship_query.get_single() {
            style.width = Val::Percent((ship_health.value / ship_health.max_value)*100.);
        }
    }
}

fn game_over(
    mut game_over_event_reader: EventReader<GameOverEvent>,
    mut game_over_text_query: Query<&mut Text, With<GameOverText>>,
) {
    for _ in game_over_event_reader.read() {
        for mut text in &mut game_over_text_query {
            text.sections[0].value = "Game Over!".into();
        }
    }
}