use bevy::prelude::*;

use super::super::health::{ChangeHealthEvent, ChangeHealthMode, HealthRunoutEvent};

#[derive(Component)]
pub struct Shield {
    last_damaged: f32,
    pub disabled: bool,
}

impl Default for Shield {
    fn default() -> Self {
        Self {
            last_damaged: 0.,
            disabled: false,
        }
    }
}

pub fn check_shield_runout(
    mut health_runout_event_reader: EventReader<HealthRunoutEvent>,
    mut shield_query: Query<&mut Shield>,
) {
    for event in health_runout_event_reader.read() {
        if let Ok(mut shield) = shield_query.get_mut(event.0) {
            shield.disabled = true;
        }
    }
}

pub fn take_damage(
    mut change_health_event_reader: EventReader<ChangeHealthEvent>,
    mut shield_query: Query<&mut Shield>,
    time: Res<Time>,
) {
    for event in change_health_event_reader.read() {
        if event.change_health_mode == ChangeHealthMode::Damage {
            if let Ok(mut shield) = shield_query.get_mut(event.entity) {
                println!("timer reset");
                shield.last_damaged = time.elapsed_seconds_wrapped();
            }
        }
    }
}

const RECHARGE_DELAY: f32 = 3.;
const RECHARGE_RATE: f32 = 10.;

pub fn refill(
    shield_query: Query<(Entity, &Shield)>,
    time: Res<Time>,
    mut change_health_event_writer: EventWriter<ChangeHealthEvent>,
) {
    for (shield_entity, shield) in shield_query.iter() {
        if time.elapsed_seconds_wrapped() - shield.last_damaged > RECHARGE_DELAY {
            change_health_event_writer.send(ChangeHealthEvent::new(
                RECHARGE_RATE * time.delta_seconds(),
                ChangeHealthMode::Heal,
                shield_entity,
            ))
        }
    }
}
