use bevy::prelude::*;

use crate::state::GameState;

#[derive(Component)]
enum PlayerDirection {
    Up,
    Down,
    None,
}

#[derive(Resource, Default)]
struct PlayerLoaded {
    pub loaded: bool,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLoaded>().add_systems(
            Update,
            (setup, move_player, handle_input).run_if(in_state(GameState::GamePlay)),
        );
    }
}

fn setup(
    mut commands: Commands,
    mut player_loaded: ResMut<PlayerLoaded>,
    player_anim: Res<crate::animation::PlayerAnimation>,
) {
    if player_loaded.loaded || !player_anim.loaded {
        return;
    }
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: player_anim.player.get_handle().unwrap(),
            transform: Transform::from_translation(Vec3::new(-350.0, 0.0, 0.0))
                .with_scale(Vec3::splat(2.0)),
            ..default()
        },
        player_anim.player.clone(),
        PlayerDirection::None,
    ));
    player_loaded.loaded = true;
}

fn move_player(time: Res<Time>, mut player_pos: Query<(&PlayerDirection, &mut Transform)>) {
    for (dir, mut transform) in &mut player_pos {
        match *dir {
            PlayerDirection::Up => transform.translation.y += 150.0 * time.delta_seconds(),
            PlayerDirection::Down => transform.translation.y -= 150.0 * time.delta_seconds(),
            _ => {}
        }
    }
}

fn handle_input(input: Res<Input<KeyCode>>, mut player: Query<&mut PlayerDirection>) {
    let dir = player.get_single_mut();
    if let Ok(mut dir) = dir {
        if input.pressed(KeyCode::W) {
            *dir = PlayerDirection::Up
        } else if input.pressed(KeyCode::S) {
            *dir = PlayerDirection::Down
        } else {
            *dir = PlayerDirection::None
        }
    }
}
