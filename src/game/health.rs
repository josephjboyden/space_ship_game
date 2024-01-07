use bevy::prelude::*;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthRunoutEvent>()
            .add_event::<ChangeHealthEvent>()
            .configure_sets(
                Update,
                (HealthSet::Write, HealthSet::Change, HealthSet::Read).chain(),
            )
            .add_systems(Update, change_health.in_set(HealthSet::Change));
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum HealthSet {
    Write,
    Change,
    Read,
}

#[derive(Component)]
pub struct Health {
    pub value: f32,
    pub max_value: f32,
}

impl Health {
    pub fn new(max_value: f32) -> Self {
        Self {
            value: max_value,
            max_value: max_value,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ChangeHealthMode {
    Damage,
    Heal,
    Set,
}

#[derive(Event)]
pub struct ChangeHealthEvent {
    value: f32,
    pub change_health_mode: ChangeHealthMode,
    pub entity: Entity,
}

impl ChangeHealthEvent {
    pub fn new(value: f32, change_health_mode: ChangeHealthMode, entity: Entity) -> Self {
        Self {
            value: value,
            change_health_mode: change_health_mode,
            entity: entity,
        }
    }
}

#[derive(Event)]
pub struct HealthRunoutEvent(pub Entity);

pub fn change_health(
    mut change_health_event_reader: EventReader<ChangeHealthEvent>,
    mut health_query: Query<&mut Health>,
    mut health_runout_event_writer: EventWriter<HealthRunoutEvent>,
) {
    for event in change_health_event_reader.read() {
        if let Ok(mut health) = health_query.get_mut(event.entity) {
            match event.change_health_mode {
                ChangeHealthMode::Damage => {
                    health.value -= event.value;
                    if health.value <= 0. {
                        health_runout_event_writer.send(HealthRunoutEvent(event.entity))
                    }
                }
                ChangeHealthMode::Heal => {
                    health.value += event.value;
                    if health.value > health.max_value {
                        health.value = health.max_value
                    }
                }
                ChangeHealthMode::Set => {
                    health.value = event.value;
                    if health.value <= 0. {
                        health_runout_event_writer.send(HealthRunoutEvent(event.entity))
                    } else if health.value > health.max_value {
                        health.value = health.max_value
                    }
                }
            }
        }
    }
}
