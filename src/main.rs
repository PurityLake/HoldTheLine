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

#[derive(Resource)]
pub struct GameplayStart {
    pub camera_endpos: Vec3,
    pub player_endpos: Vec3,
    pub camera_inplace: bool,
    pub play_inplace: bool,
}

impl GameplayStart {
    pub fn can_start(&self) -> bool {
        self.camera_inplace && self.play_inplace
    }
}

impl Default for GameplayStart {
    fn default() -> Self {
        Self {
            camera_endpos: Vec3::new(500.0, 0.0, 0.0),
            player_endpos: Vec3::new(150.0, 0.0, 0.0),
            camera_inplace: false,
            play_inplace: false,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(GameplayStart::default())
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
        .add_systems(
            Update,
            transition_to_gameplay.run_if(in_state(GameState::TransitionToGamePlay)),
        )
        .run();
}

fn transition_to_gameplay(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut gameplay_start: ResMut<GameplayStart>,
    mut next_state: ResMut<NextState<GameState>>,
    mut camera: Query<(&Camera2d, &mut Transform)>,
) {
    if !gameplay_start.camera_inplace {
        for (_, mut transform) in camera.iter_mut() {
            transform.translation.x += 200.0 * time.delta_seconds();
            if transform.translation.x >= gameplay_start.camera_endpos.x {
                gameplay_start.camera_inplace = true;
            }
        }
    }
    if gameplay_start.can_start() {
        next_state.set(state.transition());
    }
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

fn main_menu_input(
    input: Res<Input<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if input.pressed(KeyCode::Space) {
        next_game_state.set(game_state.transition());
    }
}
