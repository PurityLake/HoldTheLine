use std::{collections::HashMap, time::Duration};

use crate::json::*;
use bevy::{prelude::*, render::view::VisibilitySystems};
use serde::Deserialize;

pub struct AnimationLoadPlugin;

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct AnimationData {
    pub width: i32,
    pub height: i32,
    pub padding_x: i32,
    pub padding_y: i32,
}

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct AnimationEntry {
    pub name: String,
    pub walk: AnimationData,
    pub die: AnimationData,
}

#[derive(Asset, TypePath, Debug, Deserialize, Default)]
pub struct AnimationListAsset {
    pub enemies: Vec<AnimationEntry>,
}

#[derive(Resource, Default)]
pub struct AnimationList {
    handle: Handle<AnimationListAsset>,
    loaded: bool,
}

#[allow(dead_code)]
#[derive(Default, Clone, PartialEq)]
pub enum AnimState {
    #[default]
    Walking,
    Dying,
    Flashing,
    Dead,
}

#[derive(Component, Clone, Default)]
pub struct AnimationComponent {
    pub walk_handle: Handle<TextureAtlas>,
    pub die_handle: Handle<TextureAtlas>,
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
        walk: Handle<TextureAtlas>,
        die: Handle<TextureAtlas>,
        name: String,
        timer: Timer,
        dying_timer: Timer,
        flashing_timer: Timer,
        max_flashes: usize,
        flash_count: usize,
    ) -> Self {
        Self {
            walk_handle: walk,
            die_handle: die,
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
}

#[derive(Resource, Default)]
pub struct EnemyAnimations {
    pub enemies: HashMap<String, AnimationComponent>,
}

impl Plugin for AnimationLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonPlugin::<AnimationListAsset> {
            extensions: vec!["animinfo.json"],
            ..default()
        })
        .init_resource::<AnimationList>()
        .init_resource::<EnemyAnimations>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            load_animations.run_if(state_exists_and_equals(crate::GameState::GamePlay)),
        )
        .add_systems(
            Update,
            animate_sprite.run_if(state_exists_and_equals(crate::GameState::GamePlay)),
        )
        .add_systems(PostUpdate, dummy.after(VisibilitySystems::CheckVisibility));
    }
}

fn dummy() {}
fn setup(asset_server: Res<AssetServer>, mut anim_list: ResMut<AnimationList>) {
    anim_list.handle = asset_server.load("sprites/list.animinfo.json");
}

fn load_animations(
    list: ResMut<AnimationList>,
    asset_server: Res<AssetServer>,
    anim_assets: ResMut<Assets<AnimationListAsset>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut enemy_anims: ResMut<EnemyAnimations>,
) {
    let anim_list = anim_assets.get(&list.handle);
    if list.loaded || anim_list.is_none() {
        return;
    }
    let mut anim_map: HashMap<String, AnimationComponent> = HashMap::new();
    for enemy in anim_list.unwrap().enemies.iter() {
        let walk_texture_handle: Handle<Image> =
            asset_server.load(format!("sprites/{0}_walk.png", enemy.name));
        let die_texture_handle: Handle<Image> =
            asset_server.load(format!("sprites/{0}_die.png", enemy.name));
        let walk_texture_atlas = TextureAtlas::from_grid(
            walk_texture_handle,
            Vec2::new(enemy.walk.width as f32, enemy.walk.height as f32),
            4,
            1,
            Some(Vec2::new(
                enemy.walk.padding_x as f32,
                enemy.walk.padding_y as f32,
            )),
            None,
        );
        let die_texture_atlas = TextureAtlas::from_grid(
            die_texture_handle,
            Vec2::new(enemy.die.width as f32, enemy.die.height as f32),
            4,
            1,
            Some(Vec2::new(
                enemy.die.padding_x as f32,
                enemy.die.padding_y as f32,
            )),
            None,
        );
        anim_map.insert(
            enemy.name.clone(),
            AnimationComponent::new(
                texture_atlases.add(walk_texture_atlas),
                texture_atlases.add(die_texture_atlas),
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
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &mut AnimationComponent,
        &mut Visibility,
    )>,
) {
    for (mut sprite, mut anim, mut visible) in &mut query {
        anim.timer.tick(time.delta());
        if anim.state == AnimState::Dying && sprite.index == anim.last {
            anim.dying_timer.tick(time.delta());
            if anim.dying_timer.just_finished() {
                anim.state = AnimState::Flashing;
            }
            continue;
        }
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
