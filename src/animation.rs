use std::{collections::HashMap, time::Duration};

use crate::{json::*, GameState};
use bevy::prelude::*;
use serde::Deserialize;

pub struct AnimationLoadPlugin;

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct TilesetData {
    pub width: i32,
    pub height: i32,
    pub padding_x: i32,
    pub padding_y: i32,
}

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct EnemyAnimationEntry {
    pub name: String,
    pub anim_names: Vec<String>,
}

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct PlayerAnimationEntry {
    pub name: String,
    pub anim_names: Vec<String>,
}

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct AnimationListAsset {
    pub enemy_anim_names: Vec<String>,
    pub player_anim_names: Vec<String>,
    pub tileset: TilesetData,
    pub enemies: Vec<EnemyAnimationEntry>,
    pub player: PlayerAnimationEntry,
}

#[derive(Resource, Default)]
pub struct AnimationList {
    handle: Handle<AnimationListAsset>,
    loaded_enemies: bool,
    loaded_players: bool,
}

impl AnimationList {
    pub fn is_loaded(&self) -> bool {
        self.loaded_enemies && self.loaded_players
    }
}

#[allow(dead_code)]
#[derive(Default, Clone, Copy, PartialEq)]
pub enum AnimState {
    #[default]
    Walking,
    Idle,
    Hurting,
    Dying,
    Flashing,
    Dead,
}

impl AnimState {
    pub fn should_anim(&self) -> bool {
        match self {
            AnimState::Walking => true,
            AnimState::Idle => true,
            AnimState::Dying => true,
            AnimState::Hurting => true,
            AnimState::Flashing => false,
            AnimState::Dead => false,
        }
    }

    pub fn is_dying(&self) -> bool {
        matches!(
            self,
            AnimState::Dying | AnimState::Dead | AnimState::Flashing
        )
    }
}

impl ToString for AnimState {
    fn to_string(&self) -> String {
        match self {
            AnimState::Walking => "walk".to_string(),
            AnimState::Idle => "idle".to_string(),
            AnimState::Dying => "die".to_string(),
            AnimState::Hurting => "hurt".to_string(),
            AnimState::Flashing => "flash".to_string(),
            AnimState::Dead => "dead".to_string(),
        }
    }
}

#[derive(Default)]
pub struct AnimationHandles {
    handles: HashMap<String, Handle<TextureAtlas>>,
}

impl AnimationHandles {
    pub fn get_handle(&self, state: AnimState) -> Option<Handle<TextureAtlas>> {
        if state.should_anim() {
            Some(self.handles.get(&state.to_string()).unwrap().clone())
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct AnimationComponent {
    pub first: usize,
    pub last: usize,
    pub timer: Timer,
    pub dying_timer: Timer,
    pub flashing_timer: Timer,
    pub max_flashes: usize,
    pub flash_count: usize,
    pub state: AnimState,
}

impl AnimationComponent {
    pub fn new(state: AnimState) -> Self {
        Self {
            state,
            ..Default::default()
        }
    }
}

impl Default for AnimationComponent {
    fn default() -> Self {
        Self {
            first: 0,
            last: 3,
            timer: Timer::new(Duration::from_secs_f32(0.1), TimerMode::Repeating),
            dying_timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
            flashing_timer: Timer::new(Duration::from_secs_f32(0.2), TimerMode::Repeating),
            max_flashes: 6,
            flash_count: 0,
            state: AnimState::default(),
        }
    }
}

#[derive(Resource, Default)]
pub struct ImagesToLoad {
    pub images: Vec<AssetId<Image>>,
}

#[derive(Resource, Default)]
pub struct EnemyAnimations {
    pub enemies: HashMap<String, AnimationHandles>,
}

#[derive(Resource, Default)]
pub struct PlayerAnimation {
    pub loaded: bool,
    pub anims: AnimationHandles,
}

impl Plugin for AnimationLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonPlugin::<AnimationListAsset> {
            extensions: vec!["animinfo.json"],
            ..default()
        })
        .init_resource::<AnimationList>()
        .init_resource::<EnemyAnimations>()
        .init_resource::<PlayerAnimation>()
        .init_resource::<ImagesToLoad>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (load_enemy_animations, load_player_animations, stop_waiting)
                .run_if(in_state(GameState::Loading)),
        )
        .add_systems(
            Update,
            wait_for_assets_to_load.run_if(in_state(GameState::Waiting)),
        )
        .add_systems(
            Update,
            (animate_sprite, flash_sprite).run_if(not(in_state(GameState::Pause))),
        );
    }
}

fn setup(mut list: ResMut<AnimationList>, asset_server: Res<AssetServer>) {
    if list.is_loaded() {
        return;
    }
    list.handle = asset_server.load("sprites/list.animinfo.json");
}

fn stop_waiting(
    list: ResMut<AnimationList>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if list.is_loaded() {
        next_state.set(state.transition());
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
        for name in anim_list.enemy_anim_names.iter() {
            let texture_handle: Handle<Image> =
                asset_server.load(format!("sprites/enemies/{0}_{1}.png", enemy.name, name));
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
            image_handles.insert(name.clone(), texture_atlases.add(texture_atlas));
        }
        anim_map.insert(
            enemy.name.clone(),
            AnimationHandles {
                handles: image_handles,
            },
        );
    }
    enemy_anims.enemies = anim_map;
    list.loaded_enemies = true;
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
    let mut image_handles: HashMap<String, Handle<TextureAtlas>> = HashMap::new();
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
        image_handles.insert(name.clone(), texture_atlases.add(texture_atlas));
    }
    player_anim.anims.handles = image_handles;
    player_anim.loaded = true;
    list.loaded_players = true;
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &mut AnimationComponent)>,
) {
    for (mut sprite, mut anim) in &mut query {
        if anim.state.should_anim() {
            anim.timer.tick(time.delta());
            if anim.state == AnimState::Dying && sprite.index == anim.last {
                anim.dying_timer.tick(time.delta());
                if anim.dying_timer.just_finished() {
                    anim.state = AnimState::Flashing;
                }
                continue;
            }
            if anim.timer.just_finished() {
                sprite.index = if sprite.index == anim.last {
                    anim.first
                } else {
                    sprite.index + 1
                };
            }
        }
    }
}

fn flash_sprite(time: Res<Time>, mut query: Query<(&mut AnimationComponent, &mut Visibility)>) {
    for (mut anim, mut visible) in &mut query {
        if anim.state == AnimState::Flashing {
            anim.flashing_timer.tick(time.delta());
            if anim.flashing_timer.just_finished() {
                anim.flash_count += 1;
                match *visible {
                    Visibility::Visible => *visible = Visibility::Hidden,
                    Visibility::Hidden => *visible = Visibility::Visible,
                    Visibility::Inherited => (),
                }
                if anim.flash_count >= anim.max_flashes {
                    anim.state = AnimState::Dead;
                }
            }
        }
    }
}

fn wait_for_assets_to_load(
    mut events: EventReader<AssetEvent<Image>>,
    mut images_to_load: ResMut<ImagesToLoad>,
    anim_list: Res<AnimationList>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if anim_list.is_loaded() {
        println!("{} assets to load", images_to_load.images.len());
        for event in events.read() {
            if let AssetEvent::LoadedWithDependencies { id } = event {
                if images_to_load.images.contains(id) {
                    images_to_load.images.retain(|x| x != id);
                }
            }
        }
        if images_to_load.images.is_empty() {
            next_state.set(state.transition());
        }
    }
}
