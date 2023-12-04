use crate::json::*;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component)]
pub struct AnimationIndicies {
    pub first: usize,
    pub last: usize,
}

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

impl Plugin for AnimationLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonPlugin::<AnimationListAsset> {
            extensions: vec!["animinfo.json"],
            ..default()
        })
        .init_resource::<AnimationList>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(crate::GameState::GamePlay), load_animations);
    }
}

fn setup(asset_server: Res<AssetServer>, mut anim_list: ResMut<AnimationList>) {
    anim_list.handle = asset_server.load("sprites/list.animinfo.json");
}

fn load_animations(list: ResMut<AnimationList>, anim_assets: ResMut<Assets<AnimationListAsset>>) {
    let anim_list = anim_assets.get(&list.handle);
    if list.loaded || anim_list.is_none() {
        return;
    }
    println!("{:?}", anim_list.unwrap());
}
