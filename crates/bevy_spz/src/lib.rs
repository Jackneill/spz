// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Bevy plugin for loading .SPZ (Gaussian Splatting) assets.
//!
//! Initially based on:
//! https://github.com/AmionSky/bevy_obj
//! https://github.com/nilclass/bevy_stl

use bevy::asset::Asset;
use bevy::asset::AssetApp;
use bevy::{
	app::{App, Plugin},
	reflect::TypePath,
};

pub mod asset;

#[derive(Default)]
pub struct SpzPlugin;

impl Plugin for SpzPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset_loader::<asset::SpzLoader>();
	}
}

#[derive(Asset, TypePath)]
#[repr(transparent)]
pub struct GaussianSplat(spz::gaussian_splat::GaussianSplat);

pub const EXTENSIONS: &[&str] = &["spz", "SPZ"];
