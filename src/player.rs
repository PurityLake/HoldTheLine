use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{AnimState, AnimationComponent, PlayerAnimation},
    state::GameState,
};

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

#[derive(Resource)]
struct PlayerAttackTimer {
    pub timer: Timer,
    pub attacked: bool,
}

impl Default for PlayerAttackTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(0.75), TimerMode::Once),
            attacked: false,
        }
    }
}

#[derive(Component, Default)]
struct PlayerAttack;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLoaded>()
            .insert_resource(PlayerAttackTimer::default())
            .add_systems(
                Update,
                (
                    setup,
                    move_player,
                    handle_input,
                    change_player_anim,
                    update_attack,
                    tick_attack_timer,
                )
                    .run_if(in_state(GameState::GamePlay)),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut player_loaded: ResMut<PlayerLoaded>,
    player_anim: Res<PlayerAnimation>,
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
        RigidBody::KinematicPositionBased,
        Collider::cuboid(6.0, 7.0),
        Sensor,
        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
    ));
    player_loaded.loaded = true;
}

fn move_player(
    time: Res<Time>,
    player_loaded: Res<PlayerLoaded>,
    player_anim: Res<PlayerAnimation>,
    mut player_pos: Query<(&PlayerDirection, &mut Transform)>,
) {
    if !player_loaded.loaded || !player_anim.loaded {
        return;
    }
    for (dir, mut transform) in &mut player_pos {
        match *dir {
            PlayerDirection::Up => transform.translation.y += 150.0 * time.delta_seconds(),
            PlayerDirection::Down => transform.translation.y -= 150.0 * time.delta_seconds(),
            _ => {}
        }
    }
}

fn change_player_anim(
    player_loaded: Res<PlayerLoaded>,
    mut player: Query<(
        &PlayerDirection,
        &mut Handle<TextureAtlas>,
        &TextureAtlasSprite,
        &mut AnimationComponent,
    )>,
) {
    if !player_loaded.loaded {
        return;
    }
    if let Ok((dir, mut handle, sprite, mut anim)) = player.get_single_mut() {
        if sprite.index == anim.last {
            match *dir {
                PlayerDirection::Up | PlayerDirection::Down => {
                    anim.state = AnimState::Walking;
                    *handle = anim.get_handle().unwrap();
                }
                _ => {
                    anim.state = AnimState::Idle;
                    *handle = anim.get_handle().unwrap();
                }
            }
        }
    }
}

fn tick_attack_timer(time: Res<Time>, mut timer: ResMut<PlayerAttackTimer>) {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        timer.attacked = false;
    }
}

fn handle_input(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<Input<KeyCode>>,
    mut player_attack_timer: ResMut<PlayerAttackTimer>,
    mut player: Query<(&mut PlayerDirection, &Transform)>,
) {
    let query = player.get_single_mut();
    if let Ok((mut dir, transform)) = query {
        if input.pressed(KeyCode::W) {
            *dir = PlayerDirection::Up
        } else if input.pressed(KeyCode::S) {
            *dir = PlayerDirection::Down
        } else {
            *dir = PlayerDirection::None
        }

        if input.pressed(KeyCode::Space) && !player_attack_timer.attacked {
            player_attack_timer.attacked = true;
            player_attack_timer.timer.reset();
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("sprites/other/player_attack.png"),
                    transform: Transform::from_translation(Vec3::new(
                        transform.translation.x + 5.0,
                        transform.translation.y,
                        0.0,
                    ))
                    .with_scale(Vec3::splat(0.75)),
                    ..default()
                },
                PlayerAttack,
                RigidBody::KinematicPositionBased,
                Collider::capsule_y(10.0, 6.0),
                Sensor,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            ));
        }
    }
}

fn update_attack(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &PlayerAttack)>,
) {
    for (entity, mut transform, _) in &mut query {
        transform.scale = transform
            .scale
            .lerp(Vec3::splat(2.0), time.delta_seconds() * 2.0);

        transform.translation.x += 150.0 * time.delta_seconds();
        if transform.translation.x > 450.0 {
            commands.entity(entity).despawn();
        }
    }
}
