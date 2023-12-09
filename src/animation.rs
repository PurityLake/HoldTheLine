use std::{collections::HashMap, time::Duration};

use crate::json::*;
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

#[allow(dead_code)]
#[derive(Default, Clone, PartialEq)]
pub enum AnimState {
    #[default]
    Walking,
    Hurting,
    Dying,
    Flashing,
    Dead,
}

impl AnimState {
    fn should_anim(&self) -> bool {
        match self {
            AnimState::Walking => true,
            AnimState::Dying => true,
            AnimState::Hurting => true,
            AnimState::Flashing => false,
            AnimState::Dead => false,
        }
    }
}

impl ToString for AnimState {
    fn to_string(&self) -> String {
        match self {
            AnimState::Walking => "walk".to_string(),
            AnimState::Dying => "die".to_string(),
            AnimState::Hurting => "hurt".to_string(),
            AnimState::Flashing => "flash".to_string(),
            AnimState::Dead => "dead".to_string(),
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct AnimationComponent {
    pub image_handles: HashMap<String, Handle<TextureAtlas>>,
    pub first: usize,
    pub last: usize,
    pub name: String,
    pub timer: Timer,
    pub dying_timer: Timer,
    pub flashing_timer: Timer,
    pub max_flashes: usize,
    pub flash_count: usize,
    pub state: AnimState,
}

impl AnimationComponent {
    fn new(
        image_handles: HashMap<String, Handle<TextureAtlas>>,
        name: String,
        timer: Timer,
        dying_timer: Timer,
        flashing_timer: Timer,
        max_flashes: usize,
        flash_count: usize,
    ) -> Self {
        Self {
            image_handles,
            first: 0,
            last: 3,
            name: name.clone(),
            timer,
            dying_timer,
            flashing_timer,
            max_flashes,
            flash_count,
            state: AnimState::default(),
        }
    }

    pub fn get_handle(&self) -> Option<Handle<TextureAtlas>> {
        if self.state.should_anim() {
            Some(
                self.image_handles
                    .get(&self.state.to_string())
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        }
    }
}

#[derive(Resource, Default)]
pub struct EnemyAnimations {
    pub enemies: HashMap<String, AnimationComponent>,
}

#[derive(Resource, Default)]
pub struct PlayerAnimation {
    pub player: AnimationComponent,
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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (load_player_animations, load_enemy_animations)
                .run_if(state_exists_and_equals(crate::GameState::GamePlay)),
        )
        .add_systems(
            Update,
            (animate_sprite, flash_sprite)
                .run_if(state_exists_and_equals(crate::GameState::GamePlay)),
        );
    }
}

fn setup(asset_server: Res<AssetServer>, mut anim_list: ResMut<AnimationList>) {
    anim_list.handle = asset_server.load("sprites/list.animinfo.json");
}

fn load_enemy_animations(
    mut list: ResMut<AnimationList>,
    asset_server: Res<AssetServer>,
    anim_assets: ResMut<Assets<AnimationListAsset>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut enemy_anims: ResMut<EnemyAnimations>,
) {
    let anim_list = anim_assets.get(&list.handle);
    if list.loaded_enemies || anim_list.is_none() {
        return;
    }
    let anim_list = anim_list.unwrap();
    let mut anim_map: HashMap<String, AnimationComponent> = HashMap::new();
    for enemy in anim_list.enemies.iter() {
        let mut image_handles: HashMap<String, Handle<TextureAtlas>> = HashMap::new();
        for name in anim_list.enemy_anim_names.iter() {
            let texture_handle: Handle<Image> =
                asset_server.load(format!("sprites/{0}_{1}.png", enemy.name, name));
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
            AnimationComponent::new(
                image_handles,
                enemy.name.clone(),
                Timer::new(Duration::from_secs_f32(0.1), TimerMode::Repeating),
                Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
                Timer::new(Duration::from_secs_f32(0.2), TimerMode::Repeating),
                6,
                0,
            ),
        );
    }
    enemy_anims.enemies = anim_map;
    list.loaded_enemies = true;
}

fn load_player_animations(
    mut list: ResMut<AnimationList>,
    asset_server: Res<AssetServer>,
    anim_assets: ResMut<Assets<AnimationListAsset>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut player_anim: ResMut<PlayerAnimation>,
) {
    let anim_list = anim_assets.get(&list.handle);
    if list.loaded_players || anim_list.is_none() {
        return;
    }
    let anim_list = anim_list.unwrap();
    let mut image_handles: HashMap<String, Handle<TextureAtlas>> = HashMap::new();
    let player = &anim_list.player;
    for name in anim_list.player_anim_names.iter() {
        let texture_handle: Handle<Image> =
            asset_server.load(format!("sprites/player/hero_{0}.png", name));
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
    player_anim.player = AnimationComponent::new(
        image_handles,
        player.name.clone(),
        Timer::new(Duration::from_secs_f32(0.1), TimerMode::Repeating),
        Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
        Timer::new(Duration::from_secs_f32(0.2), TimerMode::Repeating),
        6,
        0,
    );
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
