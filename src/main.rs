// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod enemy;
mod player;
mod state;

use bevy::{prelude::*, window::WindowTheme};
use enemy::EnemySpawnPlugin;
use player::PlayerPlugin;
use state::GameState;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Hold The Line".into(),
                    resolution: (800.0, 600.0).into(),
                    resizable: false,
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
            PlayerPlugin,
            EnemySpawnPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, main_menu_input)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main_menu_input(input: Res<Input<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if input.pressed(KeyCode::Space) {
        game_state.set(GameState::GamePlay);
    }
}
