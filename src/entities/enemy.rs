use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::{
    animation::{
        AnimState, AnimationComponent, AnimationHandles, AnimationList, AnimationListAsset,
        EnemyAnimations, ImagesToLoad,
    },
    data::state::GameState,
    entities::player::GameStats,
    GameplayStart,
};

pub struct EnemySpawnPlugin;

#[derive(Component)]
pub struct Enemy {
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
        app.insert_resource(EnemySpawnData::default())
            .add_systems(
                Update,
                (
                    move_enemies,
                    spawn_enemy,
                    remove_enemies,
                    react_to_collision,
                )
                    .run_if(in_state(GameState::GamePlay)),
            )
            .add_systems(
                Update,
                load_enemy_animations.run_if(in_state(GameState::Loading)),
            );
    }
}

fn spawn_enemy(
    time: Res<Time>,
    mut commands: Commands,
    mut spawn_data: ResMut<EnemySpawnData>,
    mut status: ResMut<GameStats>,
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
        status.entites_spawned += 1;
    }
}

fn move_enemies(
    mut commands: Commands,
    time: Res<Time>,
    camerapos: Res<GameplayStart>,
    mut stats: ResMut<GameStats>,
    mut enemies: Query<(Entity, &Enemy, &mut Transform, &AnimationComponent)>,
) {
    for (entity, enemy, mut transform, anim) in enemies.iter_mut() {
        if anim.state == AnimState::Walking {
            transform.translation.x -= enemy.speed * time.delta_seconds();
            if transform.translation.x <= camerapos.camera_endpos.x - 450.0 {
                commands.entity(entity).despawn();
                stats.villagers_lost += 1;
            }
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
                // player attack enemy
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

fn load_enemy_animations(
    mut list: ResMut<AnimationList>,
    asset_server: Res<AssetServer>,
    anim_assets: ResMut<Assets<AnimationListAsset>>,
    mut images_to_load: ResMut<ImagesToLoad>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut enemy_anims: ResMut<EnemyAnimations>,
) {
    if !asset_server.is_loaded_with_dependencies(&list.handle) {
        return;
    }
    let anim_list = anim_assets.get(&list.handle);
    let anim_list = anim_list.unwrap();
    let mut anim_map: HashMap<String, AnimationHandles> = HashMap::new();
    for enemy in anim_list.enemies.iter() {
        let mut image_handles: HashMap<String, Handle<TextureAtlas>> = HashMap::new();
        for name in enemy.anim_names.iter() {
            let texture_handle: Handle<Image> =
                asset_server.load(format!("sprites/enemies/{0}_{1}.png", enemy.name, name));
            images_to_load.images.push(texture_handle.id());
            let texture_atlas = TextureAtlas::from_grid(
                texture_handle,
                Vec2::new(anim_list.tileset.width as f32, enemy.height),
                4,
                1,
                Some(Vec2::new(
                    anim_list.tileset.padding_x as f32,
                    anim_list.tileset.padding_y as f32,
                )),
                None,
            );
            image_handles.insert(name.clone(), texture_atlases.add(texture_atlas));
        }
        anim_map.insert(enemy.name.clone(), AnimationHandles::new(image_handles));
    }
    enemy_anims.enemies = anim_map;
    list.loaded_enemies = true;
}
