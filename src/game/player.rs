use bevy::{
    input::{gamepad::GamepadButtonInput, ButtonState},
    prelude::*,
    utils::HashMap,
};
use num::FromPrimitive;
use num_derive::FromPrimitive;
use std::mem;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerManager::default())
            .add_event::<PlayerJoinedEvent>()
            .add_systems(PreUpdate, (spawn_controller_player, spawn_keyboard_player))
            .add_systems(Update, (player_joined,));
    }
}

#[derive(FromPrimitive, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Role {
    Pilot,
    Gunner,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct InputScheme {
    keyboard: bool,
    id: Option<usize>,
}

impl InputScheme {
    fn from_gamepad(gamepad: Gamepad) -> Self {
        Self {
            keyboard: false,
            id: Some(gamepad.id),
        }
    }

    pub const KEYBOARD: Self = InputScheme {
        keyboard: true,
        id: None,
    };

    pub fn is_controller(&self) -> bool {
        return !self.keyboard;
    }

    pub fn is_keyboard(&self) -> bool {
        return self.keyboard;
    }
}

#[derive(Resource, Debug)]
pub struct PlayerManager {
    player_count: u32,
    available_roles: Vec<bool>,
    used_schemes: Vec<InputScheme>,
    scheme_lookup: HashMap<Role, InputScheme>,
}

impl Default for PlayerManager {
    fn default() -> Self {
        Self {
            player_count: 0,
            available_roles: vec![true; mem::variant_count::<Role>()], //variant_count is unstable features
            used_schemes: vec![],
            scheme_lookup: HashMap::default(),
        }
    }
}
impl PlayerManager {
    fn add_player(&mut self, role: Role, scheme: InputScheme) {
        self.player_count += 1;
        self.available_roles[role as usize] = false;
        self.used_schemes.push(scheme);
        self.scheme_lookup.insert(role, scheme);
    }

    pub fn get_input_scheme(&self, role: Role) -> Option<&InputScheme> {
        self.scheme_lookup.get(&role)
    }
}

#[derive(Event)]
struct PlayerJoinedEvent(Entity);

#[derive(Component)]
struct Player {
    role: Role,
    input_scheme: InputScheme,
}

impl Player {
    fn new(role: Role, input_scheme: InputScheme) -> Player {
        Player {
            role: role,
            input_scheme: input_scheme,
        }
    }
}

fn spawn_keyboard_player(
    mut player_manager: ResMut<PlayerManager>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut player_joined_event_writer: EventWriter<PlayerJoinedEvent>,
) {
    if keys.just_pressed(KeyCode::Return) {
        if !player_manager.used_schemes.contains(&InputScheme::KEYBOARD) {
            spawn_player(
                &mut player_manager,
                &mut commands,
                InputScheme::KEYBOARD,
                &mut player_joined_event_writer,
            );
        }
    }
}

fn spawn_controller_player(
    mut player_manager: ResMut<PlayerManager>,
    mut commands: Commands,
    mut button_events: EventReader<GamepadButtonInput>,
    mut player_joined_event_writer: EventWriter<PlayerJoinedEvent>,
) {
    for button_event in button_events.read() {
        let scheme = InputScheme::from_gamepad(button_event.button.gamepad);
        if button_event.button.button_type == GamepadButtonType::South
            && button_event.state == ButtonState::Pressed
            && !player_manager.used_schemes.contains(&scheme)
        {
            spawn_player(
                &mut player_manager,
                &mut commands,
                scheme,
                &mut player_joined_event_writer,
            );
        }
    }
}

fn spawn_player(
    player_manager: &mut ResMut<PlayerManager>,
    commands: &mut Commands,
    scheme: InputScheme,
    player_joined_event_writer: &mut EventWriter<PlayerJoinedEvent>,
) {
    for i in 0..player_manager.available_roles.len() {
        if player_manager.available_roles[i] {
            let role: Role = FromPrimitive::from_usize(i).unwrap();
            let id = commands.spawn(Player::new(role.clone(), scheme)).id();
            player_manager.add_player(role.clone(), scheme);
            player_joined_event_writer.send(PlayerJoinedEvent(id));
            break;
        }
    }
}

fn player_joined(
    mut player_joined_event_reader: EventReader<PlayerJoinedEvent>,
    player_query: Query<&Player>,
) {
    for event in player_joined_event_reader.read() {
        if let Ok(player) = player_query.get(event.0) {
            if player.input_scheme.is_keyboard() {
                println!("{:?} just joined with keyboard and mouse", player.role)
            } else {
                println!(
                    "{:?} just joined with controller {}",
                    player.role,
                    player.input_scheme.id.unwrap()
                )
            }
        } else {
            println!("joined event missed because player not yet spawned")
        }
    }
}
