use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::{
    animation::{AnimState, AnimationComponent, EnemyAnimations},
    state::GameState,
    GameplayStart,
};

pub struct EnemySpawnPlugin;

#[derive(Component)]
struct Enemy {
    pub name: String,
    speed: f32,
}

impl Enemy {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            speed: 75.0,
        }
    }
}

#[derive(Resource)]
struct EnemySpawnData {
    curr_spawned: i32,
    timer: Timer,
}

impl Default for EnemySpawnData {
    fn default() -> Self {
        Self {
            curr_spawned: 0,
            timer: Timer::new(Duration::from_secs_f32(0.1), TimerMode::Repeating),
        }
    }
}

impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnData::default()).add_systems(
            Update,
            (
                move_enemies,
                spawn_enemy,
                remove_enemies,
                react_to_collision,
            )
                .run_if(in_state(GameState::GamePlay)),
        );
    }
}

fn spawn_enemy(
    time: Res<Time>,
    mut commands: Commands,
    mut spawn_data: ResMut<EnemySpawnData>,
    gameplay_start: Res<GameplayStart>,
    enemy_anims: Res<EnemyAnimations>,
) {
    spawn_data.timer.tick(time.delta());
    if spawn_data.timer.just_finished() {
        let mut rng = thread_rng();
        let enemy_name = enemy_anims.enemies.keys().choose(&mut rng).unwrap();
        let anim = enemy_anims.enemies.get(enemy_name).unwrap();
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: anim.get_handle(AnimState::Walking).unwrap(),
                transform: Transform::from_translation(Vec3::new(
                    gameplay_start.camera_endpos.x + 450.0,
                    rng.gen_range(-250.0..250.0),
                    0.,
                ))
                .with_scale(Vec3::splat(2.0)),
                visibility: Visibility::Visible,
                ..default()
            },
            AnimationComponent::default(),
            Enemy::new(enemy_name),
            RigidBody::KinematicPositionBased,
            Collider::cuboid(6.0, 7.0),
            Sensor,
            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
        ));
        spawn_data.curr_spawned += 1;
    }
}

fn move_enemies(
    time: Res<Time>,
    mut enemies: Query<(&Enemy, &mut Transform, &AnimationComponent)>,
) {
    for (enemy, mut transform, anim) in enemies.iter_mut() {
        if anim.state == AnimState::Walking {
            transform.translation.x -= enemy.speed * time.delta_seconds();
        }
    }
}

fn remove_enemies(
    mut commands: Commands,
    enemies: Query<(Entity, &AnimationComponent)>,
    mut spawn_data: ResMut<EnemySpawnData>,
) {
    for (entity, data) in enemies.iter() {
        if data.state == AnimState::Dead {
            commands.entity(entity).despawn();
            spawn_data.curr_spawned -= 1;
        }
    }
}

fn react_to_collision(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    anims: Res<EnemyAnimations>,
    mut query: Query<(
        Entity,
        &Enemy,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
        &mut AnimationComponent,
    )>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(a, b, flags) = event {
            if flags.bits() & 0b01 == 0b01 {
                let enemy = if let Ok(result) = query.get_mut(*a) {
                    Ok(result)
                } else if let Ok(result) = query.get_mut(*b) {
                    Ok(result)
                } else {
                    Err(())
                };
                if let Ok((entity, enemy, mut handle, mut atlas, mut anim)) = enemy {
                    if !anim.state.is_dying() {
                        anim.state = AnimState::Dying;
                        atlas.index = 0;
                        *handle = anims
                            .enemies
                            .get(&enemy.name)
                            .unwrap()
                            .get_handle(AnimState::Dying)
                            .unwrap();
                        commands
                            .entity(entity)
                            .remove::<Collider>()
                            .remove::<ActiveCollisionTypes>()
                            .remove::<ActiveEvents>()
                            .remove::<CollisionGroups>();
                    }
                }
            }
        }
    }
}
