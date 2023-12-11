// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod animation;
mod enemy;
mod json;
mod player;
mod state;

use animation::AnimationLoadPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowTheme};
use bevy_rapier2d::prelude::*;
use enemy::EnemySpawnPlugin;
use player::PlayerPlugin;
use state::GameState;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
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
                })
                .set(ImagePlugin::default_nearest()),
            PlayerPlugin,
            EnemySpawnPlugin,
            AnimationLoadPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            #[cfg(debug_assertions)]
            RapierDebugRenderPlugin::default(),
        ))
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (main_menu_input).run_if(in_state(GameState::MainMenu)),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(-500.0, 0.0, 1000.0)),
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sprites/map/map.png"),
        transform: Transform::from_scale(Vec3::new(1.25, 1.25, 1.0)),
        ..default()
    });
}

fn main_menu_input(input: Res<Input<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if input.pressed(KeyCode::Space) {
        game_state.set(GameState::GamePlay);
    }
}
