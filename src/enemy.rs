use bevy::prelude::*;
use rand::prelude::*;

use crate::{animation::EnemyAnimations, state::GameState};

pub struct EnemySpawnPlugin;

#[derive(Component)]
struct Enemy {
    alive: bool,
    speed: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            alive: true,
            speed: 100.,
        }
    }
}

#[derive(Resource)]
struct EnemySpawnData {
    curr_spawned: i32,
    max_spawn: i32,
    curr_time: f32,
    spawn_time: f32,
}

impl Default for EnemySpawnData {
    fn default() -> Self {
        Self {
            curr_spawned: 0,
            max_spawn: 32,
            curr_time: 0.,
            spawn_time: 1.,
        }
    }
}

impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<EnemySpawnData>(EnemySpawnData::default())
            .add_systems(
                Update,
                (move_enemies, spawn_enemy, remove_enemies).run_if(in_state(GameState::GamePlay)),
            );
    }
}

fn spawn_enemy(
    time: Res<Time>,
    mut commands: Commands,
    mut spawn_data: ResMut<EnemySpawnData>,
    enemy_anims: Res<EnemyAnimations>,
) {
    if spawn_data.curr_time > spawn_data.spawn_time {
        if spawn_data.curr_spawned + 1 < spawn_data.max_spawn {
            let mut rng = thread_rng();
            let anim = enemy_anims.enemies.get("demon").unwrap();
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: anim.walk_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        500.,
                        rng.gen_range(-250.0..250.0),
                        0.,
                    ))
                    .with_scale(Vec3::splat(3.0)),
                    ..default()
                },
                anim.clone(),
                Enemy::default(),
            ));
            spawn_data.curr_spawned += 1;
            spawn_data.curr_time = 0.;
        }
    } else {
        spawn_data.curr_time += time.delta_seconds();
    }
}

fn move_enemies(time: Res<Time>, mut enemies: Query<(&mut Enemy, &mut Transform)>) {
    for (mut enemy, mut transform) in enemies.iter_mut() {
        if enemy.alive {
            transform.translation.x -= enemy.speed * time.delta_seconds();
            if transform.translation.x < 0. {
                enemy.alive = false;
            }
        }
    }
}

fn remove_enemies(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy)>,
    mut spawn_data: ResMut<EnemySpawnData>,
) {
    for (entity, data) in enemies.iter() {
        if !data.alive {
            commands.entity(entity).despawn();
            spawn_data.curr_spawned -= 1;
        }
    }
}
