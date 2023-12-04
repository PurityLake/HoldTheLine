use std::marker::PhantomData;

use crate::json::*;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component)]
pub struct AnimationIndicies {
    pub first: usize,
    pub last: usize,
}

pub struct AnimationLoadPlugin;

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct AnimationListAsset {
    pub enemies: Vec<String>,
    pub tile_width: i32,
    pub tile_height: i32,
    pub tile_padding: i32,
}

#[derive(Resource, Default)]
pub struct AnimationList {
    handle: Handle<AnimationListAsset>,
    loaded: bool,
}

impl Plugin for AnimationLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationList>()
            .init_asset::<AnimationListAsset>()
            .register_asset_loader(JsonAssetLoader::<AnimationListAsset> {
                marker: PhantomData,
            })
            .add_systems(Startup, setup)
            .add_systems(OnEnter(crate::GameState::GamePlay), load_animations);
    }
}

fn setup(asset_server: Res<AssetServer>, mut anim_list: ResMut<AnimationList>) {
    anim_list.handle = asset_server.load("sprites/animations.json");
}

fn load_animations(list: ResMut<AnimationList>, anim_assets: ResMut<Assets<AnimationListAsset>>) {
    let anim_list = anim_assets.get(&list.handle);
    if list.loaded || anim_list.is_none() {
        return;
    }
    println!("{:?}", anim_list.unwrap());
}
