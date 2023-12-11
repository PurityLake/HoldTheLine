use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::de;

use crate::{
    animation::{
        AnimState, AnimationComponent, AnimationList, AnimationListAsset, ImagesToLoad,
        PlayerAnimation,
    },
    state::GameState,
    GameplayStart,
};

#[derive(Resource)]
struct PlayerData {
    max_health: i32,
    health: i32,
    timer: Timer,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            max_health: 10,
            health: 10,
            timer: Timer::new(Duration::from_secs_f32(2.0), TimerMode::Repeating),
        }
    }
}

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
            timer: Timer::new(Duration::from_secs_f32(0.8), TimerMode::Once),
            attacked: false,
        }
    }
}

#[derive(Component)]
struct PlayerAttack {
    pub health: i32,
}

impl Default for PlayerAttack {
    fn default() -> Self {
        Self { health: 10 }
    }
}

#[derive(Resource, Default)]
pub struct PlayerAttackSprite {
    pub sprite: Handle<Image>,
}

pub struct PlayerPlugin;

#[derive(Resource)]
pub struct PlayerPhysicsAttached(bool);

#[derive(Resource)]
pub struct GameStats {
    pub villagers_saved: i32,
    pub villagers_lost: i32,
    pub entites_spawned: i32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            villagers_saved: 0,
            villagers_lost: 0,
            entites_spawned: 0,
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLoaded>()
            .insert_resource(PlayerAttackTimer::default())
            .insert_resource(PlayerAttackSprite::default())
            .insert_resource(PlayerPhysicsAttached(false))
            .insert_resource(GameStats::default())
            .insert_resource(PlayerData::default())
            .add_systems(Startup, load_assets)
            .add_systems(Update, setup)
            .add_systems(OnEnter(GameState::GamePlay), spawn_text)
            .add_systems(
                Update,
                slide_in_player.run_if(in_state(GameState::TransitionToGamePlay)),
            )
            .add_systems(
                Update,
                load_player_animations.run_if(in_state(GameState::Loading)),
            )
            .add_systems(Update, add_collisions.run_if(in_state(GameState::GamePlay)))
            .add_systems(
                Update,
                (
                    move_player,
                    handle_input,
                    change_player_anim,
                    update_attack,
                    tick_attack_timer,
                    react_to_player_attack_collision,
                    react_to_player_collision,
                    update_text,
                )
                    .run_if(in_state(GameState::GamePlay)),
            )
            .add_systems(Update, player_dies.run_if(in_state(GameState::GameOver)));
    }
}

#[derive(Component, Default)]
struct EntitiesText;

fn spawn_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_data: Res<PlayerData>,
) {
    commands.spawn((
        TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: format!("Entities Spawned: {}", 0),
                    style: TextStyle {
                        font: asset_server.load("fonts/plop.ttf"),
                        font_size: 25.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: format!("Player Life: {}", player_data.health),
                    style: TextStyle {
                        font: asset_server.load("fonts/plop.ttf"),
                        font_size: 25.0,
                        color: Color::WHITE,
                    },
                },
            ]),
            ..default()
        },
        EntitiesText,
    ));
}

fn update_text(
    score: Res<GameStats>,
    asset_server: Res<AssetServer>,
    player_data: Res<PlayerData>,
    mut query: Query<(&mut Text, &EntitiesText)>,
) {
    for (mut text, _) in &mut query {
        *text = Text::from_sections([
            TextSection {
                value: format!("Entities Spawned: {}", 0),
                style: TextStyle {
                    font: asset_server.load("fonts/plop.ttf"),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            },
            TextSection {
                value: format!("Player Life: {}", player_data.health),
                style: TextStyle {
                    font: asset_server.load("fonts/plop.ttf"),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            },
        ]);
    }
}

fn player_dies(
    mut command: Commands,
    player_anim: Res<PlayerAnimation>,
    mut player: Query<(
        Entity,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
        &mut AnimationComponent,
        &PlayerDirection,
    )>,
) {
    if let Ok((entity, mut handle, mut sprite, mut anim, _)) = player.get_single_mut() {
        if matches!(anim.state, AnimState::Walking | AnimState::Idle) {
            anim.state = AnimState::Dying;
            sprite.index = 0;
            *handle = player_anim.anims.get_handle(anim.state).unwrap();
        } else if matches!(anim.state, AnimState::Dead) {
            command.entity(entity).despawn();
        }
    }
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut attack_sprite: ResMut<PlayerAttackSprite>,
    mut images_to_load: ResMut<ImagesToLoad>,
) {
    let handle = asset_server.load("sprites/other/player_attack.png");
    images_to_load.images.push(handle.id());
    attack_sprite.sprite = handle;
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
            texture_atlas: player_anim.anims.get_handle(AnimState::Idle).unwrap(),
            transform: Transform::from_translation(Vec3::new(-500.0, 40.0, 0.0))
                .with_scale(Vec3::splat(2.0)),
            ..default()
        },
        AnimationComponent::new(AnimState::Idle),
        PlayerDirection::None,
    ));
    player_loaded.loaded = true;
}

fn add_collisions(
    mut commands: Commands,
    mut attached: ResMut<PlayerPhysicsAttached>,
    player: Query<Entity, With<PlayerDirection>>,
) {
    if !attached.0 {
        if let Ok(entity) = player.get_single() {
            commands.entity(entity).insert((
                RigidBody::KinematicPositionBased,
                Collider::cuboid(6.0, 7.0),
                Sensor,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
            ));
        }
        attached.0 = true;
    }
}

fn slide_in_player(
    time: Res<Time>,
    mut gameplay_start: ResMut<GameplayStart>,
    player_anim: Res<PlayerAnimation>,
    mut player: Query<(
        &PlayerDirection,
        &mut Transform,
        &mut Handle<TextureAtlas>,
        &mut AnimationComponent,
    )>,
) {
    if !gameplay_start.play_inplace || !player_anim.loaded {
        for (_, mut player_transform, mut handle, mut anim) in player.iter_mut() {
            if anim.state == AnimState::Idle {
                anim.state = AnimState::Walking;
                *handle = player_anim.anims.get_handle(anim.state).unwrap();
            }
            player_transform.translation.x += 200.0 * time.delta_seconds();
            if player_transform.translation.x >= gameplay_start.player_endpos.x {
                gameplay_start.play_inplace = true;
                anim.state = AnimState::Idle;
                *handle = player_anim.anims.get_handle(anim.state).unwrap();
            }
        }
    }
}

fn move_player(
    time: Res<Time>,
    player_anim: Res<PlayerAnimation>,
    mut player_data: ResMut<PlayerData>,
    mut player_pos: Query<(&PlayerDirection, &mut Transform)>,
) {
    if !player_anim.loaded {
        return;
    }
    for (dir, mut transform) in &mut player_pos {
        match *dir {
            PlayerDirection::Up => transform.translation.y += 250.0 * time.delta_seconds(),
            PlayerDirection::Down => transform.translation.y -= 250.0 * time.delta_seconds(),
            _ => {}
        }
        player_data.timer.tick(time.delta());
        if player_data.timer.just_finished() {
            player_data.health = (player_data.health + 1).min(player_data.max_health);
        }
    }
}

fn change_player_anim(
    player_anim: Res<PlayerAnimation>,
    mut player: Query<(
        &PlayerDirection,
        &mut Handle<TextureAtlas>,
        &TextureAtlasSprite,
        &mut AnimationComponent,
    )>,
) {
    if !player_anim.loaded {
        return;
    }
    if let Ok((dir, mut handle, sprite, mut anim)) = player.get_single_mut() {
        if sprite.index == anim.last {
            match *dir {
                PlayerDirection::Up | PlayerDirection::Down => {
                    anim.state = AnimState::Walking;
                    *handle = player_anim.anims.get_handle(anim.state).unwrap();
                }
                _ => {
                    anim.state = AnimState::Idle;
                    *handle = player_anim.anims.get_handle(anim.state).unwrap();
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
    input: Res<Input<KeyCode>>,
    player_attack: Res<PlayerAttackSprite>,
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
                    texture: player_attack.sprite.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        transform.translation.x + 5.0,
                        transform.translation.y,
                        0.0,
                    ))
                    .with_scale(Vec3::splat(0.75)),
                    visibility: Visibility::Visible,
                    ..default()
                },
                PlayerAttack::default(),
                RigidBody::KinematicPositionBased,
                Collider::capsule_y(10.0, 6.0),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                CollisionGroups::new(Group::GROUP_2, Group::GROUP_1),
            ));
        }
    }
}

fn update_attack(
    mut commands: Commands,
    time: Res<Time>,
    start: Res<GameplayStart>,
    mut query: Query<(Entity, &mut Transform, &PlayerAttack)>,
) {
    for (entity, mut transform, _) in &mut query {
        transform.scale = transform
            .scale
            .lerp(Vec3::splat(2.0), time.delta_seconds() * 2.0);

        transform.translation.x += 150.0 * time.delta_seconds();
        if transform.translation.x > start.camera_endpos.x + 450.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn react_to_player_attack_collision(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(Entity, &mut PlayerAttack)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(a, b, flags) = event {
            if flags.bits() & 0b01 == 0b01 {
                let attack = if let Ok(result) = query.get_mut(*a) {
                    Ok(result)
                } else if let Ok(result) = query.get_mut(*b) {
                    Ok(result)
                } else {
                    Err(())
                };
                if let Ok((entity, mut attack)) = attack {
                    attack.health -= 1;
                    if attack.health <= 0 {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

fn react_to_player_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_data: ResMut<PlayerData>,
    mut query: Query<(Entity, &PlayerDirection)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(a, b, flags) = event {
            if flags.bits() & 0b01 == 0b01 {
                let player = if let Ok(result) = query.get_mut(*a) {
                    Ok(result)
                } else if let Ok(result) = query.get_mut(*b) {
                    Ok(result)
                } else {
                    Err(())
                };
                if let Ok((_, _)) = player {
                    player_data.health -= 1;
                    player_data.timer.reset();
                    if player_data.health <= 0 {
                        next_state.set(GameState::GameOver);
                    }
                }
            }
        }
    }
}

fn load_player_animations(
    mut list: ResMut<AnimationList>,
    asset_server: Res<AssetServer>,
    anim_assets: ResMut<Assets<AnimationListAsset>>,
    mut images_to_load: ResMut<ImagesToLoad>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut player_anim: ResMut<PlayerAnimation>,
) {
    if !asset_server.is_loaded_with_dependencies(&list.handle) {
        return;
    }
    let anim_list = anim_assets.get(&list.handle);
    let anim_list = anim_list.unwrap();
    let player = &anim_list.player;
    for name in player.anim_names.iter() {
        let texture_handle: Handle<Image> =
            asset_server.load(format!("sprites/player/hero_{0}.png", name));
        images_to_load.images.push(texture_handle.id());
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(
                anim_list.tileset.width as f32,
                anim_list.tileset.height as f32,
            ),
            4,
            1,
            Some(Vec2::new(
                anim_list.tileset.padding_x as f32,
                anim_list.tileset.padding_y as f32,
            )),
            None,
        );
        player_anim
            .anims
            .add_handle(name.clone(), texture_atlases.add(texture_atlas));
    }
    player_anim.loaded = true;
    list.loaded_players = true;
}
