use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use super::{
    health::{ChangeHealthEvent, ChangeHealthMode},
    physics::{CircleCollider, CollisionLayerNames, CollisionLayers, UniqueCollideEvent},
    quad_tree::QuadTreeElement,
    ship::Ship,
};

pub struct HealthPackPlugin;

impl Plugin for HealthPackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnHelthPackEvent>()
            .add_systems(Update, (spawn_health_pack, pickup));
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
    mut collision_layers: ResMut<CollisionLayers>,
) {
    for event in spawn_health_pack_event_reader.read() {
        let health_pack_entity = commands
            .spawn((
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
                CircleCollider::new(20., CollisionLayerNames::HealthPacks),
                QuadTreeElement,
            ))
            .id();
        collision_layers.layers[CollisionLayerNames::HealthPacks as usize]
            .in_layer
            .push(health_pack_entity);
    }
}

fn pickup(
    mut commands: Commands,
    health_pack_query: Query<Entity, With<HealthPack>>,
    ship_query: Query<Entity, With<Ship>>,
    mut change_health_event_writer: EventWriter<ChangeHealthEvent>,
    mut unique_collision_event_reader: EventReader<UniqueCollideEvent>,
) {
    for event in unique_collision_event_reader.read() {
        if let Ok(ship) = ship_query.get(event.a) {
            if let Ok(health_pack) = health_pack_query.get(event.b) {
                commands.entity(health_pack).despawn();
                change_health_event_writer.send(ChangeHealthEvent::new(
                    1.,
                    ChangeHealthMode::Heal,
                    ship,
                ))
            }
        }
    }
}
