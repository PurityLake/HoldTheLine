use bevy::prelude::*;

use crate::state::GameState;

#[derive(Component)]
enum PlayerDirection {
    Up,
    Down,
    None,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GamePlay), setup)
            .add_systems(
                Update,
                (move_player, handle_input).run_if(in_state(GameState::GamePlay)),
            );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 0.5),
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-350.0, 0.0, 0.0)),
            ..default()
        },
        PlayerDirection::None,
    ));
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
    let mut dir = player.get_single_mut().unwrap();
    if input.pressed(KeyCode::W) {
        *dir = PlayerDirection::Up
    } else if input.pressed(KeyCode::S) {
        *dir = PlayerDirection::Down
    } else {
        *dir = PlayerDirection::None
    }
}
