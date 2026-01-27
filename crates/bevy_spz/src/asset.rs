// SPDX-License-Identifier: Apache-2.0 OR MIT

use bevy::{
	asset::{AssetLoader, LoadContext, io::Reader},
	reflect::TypePath,
	tasks::ConditionalSendFuture,
};
use serde::{Deserialize, Serialize};
use spz::{gaussian_splat::GaussianSplat, packed::PackedGaussianSplat};
use thiserror::Error;

#[derive(Default, TypePath)]
pub struct SpzLoader;

impl AssetLoader for SpzLoader {
	type Error = Error;
	type Settings = Settings;
	type Asset = crate::GaussianSplat;

	#[inline]
	fn load(
		&self,
		reader: &mut dyn Reader,
		settings: &Self::Settings,
		_load_context: &mut LoadContext,
	) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
		async move {
			let mut buf = Vec::new();

			reader.read_to_end(&mut buf).await?;

			let gs = GaussianSplat::new_from_packed_gaussians(
				&PackedGaussianSplat::from_bytes(&buf).map_err(Error::LoadError)?,
				&settings.load_opts,
			)
			.map_err(Error::LoadError)?;

			Ok(crate::GaussianSplat(gs))
		}
	}

	#[inline]
	fn extensions(&self) -> &[&str] {
		crate::EXTENSIONS
	}
}

/// SPZ asset loader settings.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Settings {
	/// Options for loading the Gaussian Splat.
	pub load_opts: spz::gaussian_splat::LoadOptions,
}

#[derive(Error, Debug)]
pub enum Error {
	#[error("io error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("failed to load SPZ asset: {0}")]
	LoadError(anyhow::Error),
}
