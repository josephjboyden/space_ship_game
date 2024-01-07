use bevy::prelude::*;

use super::{
    super::{
        physics::components::Acceleration,
        player::{PlayerManager, Role},
    },
    Ship,
};

const FORWARD_ACCELERATION: f32 = 250.;
const OTHER_ACCELERATION: f32 = 10.;
const MAX_ANGULAR_VELOCITY: f32 = 20.;

pub struct PilotPlugin;

impl Plugin for PilotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_gamepad_input,
                handle_keyboard_input,
                handle_stick_input,
                handle_mouse_input,
                set_rotation,
            ),
        );
    }
}

fn handle_gamepad_input(
    gamepads: Res<Gamepads>,
    button_axes: Res<Axis<GamepadButton>>,
    mut ship_acceleration_query: Query<&mut Acceleration, With<Ship>>,
    player_manager: ResMut<PlayerManager>,
) {
    match player_manager.get_input_scheme(Role::Pilot) {
        Some(input_scheme) => {
            if !input_scheme.is_controller() {
                return;
            };
        }
        None => return,
    }

    if let Ok(mut ship_acceleration) = ship_acceleration_query.get_single_mut() {
        for gamepad in gamepads.iter() {
            let right_trigger = button_axes
                .get(GamepadButton::new(
                    gamepad,
                    GamepadButtonType::RightTrigger2,
                ))
                .unwrap();

            ship_acceleration.value.y = FORWARD_ACCELERATION * right_trigger;
        }
    }
}

fn handle_keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut ship_acceleration_query: Query<&mut Acceleration, With<Ship>>,
    player_manager: ResMut<PlayerManager>,
) {
    match player_manager.get_input_scheme(Role::Pilot) {
        Some(input_scheme) => {
            if !input_scheme.is_keyboard() {
                return;
            };
        }
        None => return,
    }

    if let Ok(mut ship_acceleration) = ship_acceleration_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Space) {
            ship_acceleration.value.y += FORWARD_ACCELERATION
        }
        if keys.just_released(KeyCode::Space) {
            ship_acceleration.value.y -= FORWARD_ACCELERATION
        }

        if keys.just_pressed(KeyCode::W) {
            ship_acceleration.value.y += OTHER_ACCELERATION
        }
        if keys.just_released(KeyCode::W) {
            ship_acceleration.value.y -= OTHER_ACCELERATION
        }

        if keys.just_pressed(KeyCode::S) {
            ship_acceleration.value.y -= OTHER_ACCELERATION
        }
        if keys.just_released(KeyCode::S) {
            ship_acceleration.value.y += OTHER_ACCELERATION
        }

        if keys.just_pressed(KeyCode::A) {
            ship_acceleration.value.x -= OTHER_ACCELERATION
        }
        if keys.just_released(KeyCode::A) {
            ship_acceleration.value.x += OTHER_ACCELERATION
        }

        if keys.just_pressed(KeyCode::D) {
            ship_acceleration.value.x += OTHER_ACCELERATION
        }
        if keys.just_released(KeyCode::D) {
            ship_acceleration.value.x -= OTHER_ACCELERATION
        }
    }
}

fn handle_stick_input(
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    mut ship_query: Query<&mut Ship>,
    player_manager: ResMut<PlayerManager>,
) {
    match player_manager.get_input_scheme(Role::Pilot) {
        Some(input_scheme) => {
            if !input_scheme.is_controller() {
                return;
            };
        }
        None => return,
    }

    if let Ok(mut ship) = ship_query.get_single_mut() {
        for gamepad in gamepads.iter() {
            let left_stick = Vec2::new(
                axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
                    .unwrap(),
                axes.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
                    .unwrap(),
            );
            ship.target_direction = left_stick.clone();
        }
    }
}

fn handle_mouse_input(
    window_query: Query<&Window>,
    mut ship_query: Query<&mut Ship>,
    player_manager: ResMut<PlayerManager>,
) {
    match player_manager.get_input_scheme(Role::Pilot) {
        Some(input_scheme) => {
            if !input_scheme.is_keyboard() {
                return;
            };
        }
        None => return,
    }

    if let Ok(mut ship) = ship_query.get_single_mut() {
        let window = window_query.single();

        if let Some(mut mouse_position) = window.cursor_position().clone() {
            mouse_position -= Vec2 {
                x: window.width() / 2.,
                y: window.height() / 2.,
            };
            mouse_position.y *= -1.;

            ship.target_direction = mouse_position;
        }
    }
}

fn set_rotation(mut ship_query: Query<(&mut Transform, &Ship)>, time: Res<Time>) {
    if let Ok((mut transform, ship)) = ship_query.get_single_mut() {
        let target_direction = Vec3::new(ship.target_direction.x, ship.target_direction.y, 0.);

        let z = transform
            .rotation
            .mul_vec3(Vec3::Y)
            .cross(target_direction)
            .z;

        let mut angular_velocity = 0.;

        if z > 0.0001 {
            angular_velocity = MAX_ANGULAR_VELOCITY;
        } else if z < -0.0001 {
            angular_velocity = -MAX_ANGULAR_VELOCITY;
        }

        if angular_velocity != 0. {
            transform.rotate_axis(Vec3::Z, angular_velocity * time.delta_seconds());

            if z * transform
                .rotation
                .mul_vec3(Vec3::Y)
                .cross(target_direction)
                .z
                < 0.
            {
                transform.rotation = Quat::from_axis_angle(
                    Vec3::NEG_Z,
                    ship.target_direction.angle_between(Vec2::Y),
                );
            }
        }
    }
}
