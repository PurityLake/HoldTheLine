// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod animation;
mod enemy;
mod player;
mod state;

use animation::AnimationIndicies;
use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowTheme};
use enemy::EnemySpawnPlugin;
use player::PlayerPlugin;
use state::GameState;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_state::<GameState>()
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
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (main_menu_input, animate_sprite).run_if(in_state(GameState::MainMenu)),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("sprites/demon_walk.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(12., 16.),
        4,
        1,
        Some(Vec2::new(12., 0.)),
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let animation_indicies = AnimationIndicies { first: 0, last: 3 };
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indicies.first),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indicies,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndicies,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indicies, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indicies.last {
                indicies.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn main_menu_input(input: Res<Input<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if input.pressed(KeyCode::Space) {
        game_state.set(GameState::GamePlay);
    }
}
