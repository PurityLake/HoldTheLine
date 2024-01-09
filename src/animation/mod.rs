use std::{collections::HashMap, time::Duration};

use crate::data::{json::*, state::GameState};
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
    pub height: f32,
}

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct PlayerAnimationEntry {
    pub name: String,
    pub anim_names: Vec<String>,
}

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct AnimationListAsset {
    pub tileset: TilesetData,
    pub enemies: Vec<EnemyAnimationEntry>,
    pub player: PlayerAnimationEntry,
}

#[derive(Resource, Default)]
pub struct AnimationList {
    pub handle: Handle<AnimationListAsset>,
    pub loaded_enemies: bool,
    pub loaded_players: bool,
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
    pub fn new(handles: HashMap<String, Handle<TextureAtlas>>) -> Self {
        Self { handles }
    }

    pub fn get_handle(&self, state: AnimState) -> Option<Handle<TextureAtlas>> {
        if state.should_anim() {
            Some(self.handles.get(&state.to_string()).unwrap().clone())
        } else {
            None
        }
    }

    pub fn add_handle(&mut self, key: String, handle: Handle<TextureAtlas>) {
        self.handles.insert(key, handle);
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
            dying_timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating),
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
        .add_systems(Update, stop_waiting.run_if(in_state(GameState::Loading)))
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
                    Visibility::Inherited => *visible = Visibility::Hidden,
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
