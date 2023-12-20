use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use super::physics::CircleCollider;

pub struct HealthPackPlugin;

impl Plugin for HealthPackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHelthPackEvent>()
            .add_systems(Update, spawn_health_pack);
    }
}

#[derive(Component)]
struct HealthPack;

#[derive(Event)]
pub struct SpawnHelthPackEvent {
    position: Vec2,
}

impl SpawnHelthPackEvent {
    pub fn new(position: Vec2) -> Self {
        Self { position: position }
    }
}

fn spawn_health_pack(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut spawn_health_pack_event_reader: EventReader<SpawnHelthPackEvent>,
) {
    for event in spawn_health_pack_event_reader.read() {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(20.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::RED)),
                transform: Transform::from_translation(Vec3::new(
                    event.position.x,
                    event.position.y,
                    0.15,
                )),
                ..default()
            },
            HealthPack,
            CircleCollider::new(20.),
        ));
    }
}
