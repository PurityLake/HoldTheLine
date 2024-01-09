use std::marker::PhantomData;

use bevy::utils::thiserror;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Default)]
pub struct JsonPlugin<A> {
    pub extensions: Vec<&'static str>,
    pub marker: PhantomData<A>,
}

impl<A> Plugin for JsonPlugin<A>
where
    for<'a> A: Deserialize<'a> + Asset,
{
    fn build(&self, app: &mut App) {
        app.init_asset::<A>()
            .register_asset_loader(JsonAssetLoader::<A> {
                extensions: self.extensions.clone(),
                marker: PhantomData,
            });
    }
}

#[derive(Default)]
pub struct JsonAssetLoader<A> {
    pub extensions: Vec<&'static str>,
    pub marker: std::marker::PhantomData<A>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum JsonAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse JSON: {0}")]
    JsonParseError(#[from] serde_json::error::Error),
}

impl<A> AssetLoader for JsonAssetLoader<A>
where
    for<'a> A: Deserialize<'a> + Asset,
{
    type Asset = A;
    type Settings = ();
    type Error = JsonAssetLoaderError;
    fn load<'b>(
        &'b self,
        reader: &'b mut Reader,
        _settings: &'b (),
        _load_context: &'b mut LoadContext,
    ) -> BoxedFuture<'b, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let custom_asset = serde_json::de::from_slice::<A>(&bytes)?;
            Ok(custom_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}
