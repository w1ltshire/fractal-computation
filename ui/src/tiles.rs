use std::collections::HashMap;
use std::sync::Arc;
use eframe::emath::{pos2, Rect};
use eframe::epaint::textures::TextureOptions;
use egui::{ColorImage, Context};
use egui_async::Bind;
use log::debug;
use lru::LruCache;
use walkers::{Tile, TileId, TilePiece, Tiles};
use walkers::sources::Attribution;
use crate::fractal_set;

pub type Set = Vec<(f64, f64, usize)>;

#[derive(Clone)]
pub enum CachedTexture {
	Valid(Tile),
	#[allow(unused)]
	Invalid,
	Pending,
}

pub struct FractalTiles {
	pub(crate) tiles: LruCache<TileId, CachedTexture>, // public for the crate so we can clear the cache if parameters like iterations have been changed
	egui_ctx: Context,
	binds: HashMap<TileId, Bind<ColorImage, String>>,
	pub(crate) mandelbrot_set_properties: MandelbrotSetProperties,
	pub(crate) old_mandelbrot_set_properties: MandelbrotSetProperties,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct MandelbrotSetProperties {
	pub iterations: usize,
	pub exponent: usize,
	pub samples: (usize, usize)
}

impl FractalTiles {
	pub fn new(
		egui_ctx: Context,
	) -> Self {
		let tile_size_points: u32 = 256;
		let dpr = egui_ctx.pixels_per_point();
		let samples = ((tile_size_points as f32 * dpr).round() as usize, (tile_size_points as f32 * dpr).round() as usize);
		debug!("samples={samples:?}");
		let props = MandelbrotSetProperties { iterations: 255, exponent: 2, samples };
		Self {
			tiles: LruCache::unbounded(),
			egui_ctx,
			binds: HashMap::new(),
			mandelbrot_set_properties: props,
			old_mandelbrot_set_properties: props,
		}
	}

	fn load(&mut self, tile_id: TileId) -> CachedTexture {
		let load_texture = |ctx: &Context, id: TileId, set: Arc<ColorImage>| {
			let handle = ctx.load_texture(format!("{id:?}"), set, TextureOptions::default());
			CachedTexture::Valid(Tile::Raster(handle))
		};

		if let Some(tile) = self.tiles.get(&tile_id) {
			return match tile {
				CachedTexture::Valid(t) => CachedTexture::Valid(t.clone()),
				CachedTexture::Pending => {
					let bind = self.binds.get_mut(&tile_id).unwrap();
					if let Some(set) = bind.read() {
						// no visible reason why result would be `Err`? i hope
						load_texture(&self.egui_ctx, tile_id, Arc::from(set.clone().unwrap()))
					} else {
						CachedTexture::Pending
					}
				}
				CachedTexture::Invalid => CachedTexture::Invalid,
			};
		}

		// ensure a bind exists
		self.binds.entry(tile_id).or_insert_with(Bind::default);

		let bind = self.binds.get_mut(&tile_id).unwrap();
		let props = self.mandelbrot_set_properties.clone();

		if let Some(set) = bind.read_or_request(|| async move {
			Ok(fractal_set::generate_color_image(tile_id, props))
		}) {
			load_texture(&self.egui_ctx, tile_id, Arc::from(set.clone().unwrap()))
		} else {
			CachedTexture::Pending
		}
	}
}

impl Tiles for FractalTiles {
	fn at(&mut self, tile_id: TileId) -> Option<TilePiece> {
		(0..=tile_id.zoom).rev().find_map(|zoom_candidate| {
			let (donor_tile_id, uv) = interpolate_from_lower_zoom(tile_id, zoom_candidate);
			match self.load(donor_tile_id) {
				CachedTexture::Valid(texture) => Some(TilePiece::new(texture.clone(), uv)),
				CachedTexture::Invalid | CachedTexture::Pending => None,
			}
		})
	}

	fn attribution(&self) -> Attribution {
		Attribution {
			text: "Mandelbrot Set",
			url: "".into(),
			logo_dark: None,
			logo_light: None,
		}
	}

	fn tile_size(&self) -> u32 {
		256
	}
}

// https://docs.rs/walkers/latest/src/walkers/tiles.rs.html#102-106
pub(crate) fn interpolate_from_lower_zoom(tile_id: TileId, available_zoom: u8) -> (TileId, Rect) {
	assert!(tile_id.zoom >= available_zoom);

	let d_zoom = 2u32.pow((tile_id.zoom - available_zoom) as u32);

	let x = (tile_id.x / d_zoom, tile_id.x % d_zoom);
	let y = (tile_id.y / d_zoom, tile_id.y % d_zoom);

	let zoomed_tile_id = TileId {
		x: x.0,
		y: y.0,
		zoom: available_zoom,
	};

	let z = (d_zoom as f32).recip();

	let uv = Rect::from_min_max(
		pos2(x.1 as f32 * z, y.1 as f32 * z),
		pos2(x.1 as f32 * z + z, y.1 as f32 * z + z),
	);

	(zoomed_tile_id, uv)
}