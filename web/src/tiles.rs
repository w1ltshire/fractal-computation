use std::collections::HashMap;
use eframe::emath::{pos2, Pos2, Rect};
use eframe::epaint::{ColorImage, ImageDelta};
use eframe::epaint::textures::TextureOptions;
use egui::Context;
use image::{ImageBuffer, Rgba};
use lru::LruCache;
use tracing::{error, trace};
use walkers::{Style, Tile, TileId, TilePiece, Tiles};
use walkers::sources::Attribution;

#[derive(Clone)]
enum CachedTexture {
	Valid(Tile),
	Invalid,
}

pub struct FractalTiles {
	tiles: LruCache<TileId, CachedTexture>,
	egui_ctx: Context
}

impl FractalTiles {
	pub fn new(egui_ctx: Context) -> Self {
		trace!("initializing tiles");
		Self {
			tiles: LruCache::unbounded(),
			egui_ctx,
		}
	}

	fn load(&mut self, tile_id: TileId) -> CachedTexture {
		self.tiles
			.get_or_insert(tile_id, || {
				let scale = 3.0 / (1 << tile_id.zoom) as f64;
				let x_center = (tile_id.x as f64) * scale - 2.0;
				let y_center = (tile_id.y as f64) * scale - 1.5;

				let from = (x_center, y_center);
				let to = (x_center + scale, y_center + scale);

				let mut data: Vec<[u8; 4]> = vec![[0u8; 4]; 256 * 256];
				mandelbrot::cpu::mandelbrot_simd(&mut data, from, to, 256, 256, 200);

				let mut img = ImageBuffer::<Rgba<u8>, _>::new(256, 256);
				for (i, pixel) in data.iter().enumerate() {
					let x = (i as u32) % 256;
					let y = (i as u32) / 256;
					img.put_pixel(x, y, Rgba(*pixel));
				}
				let color_image = ColorImage::from_rgba_unmultiplied(
					[256usize, 256usize],
					img.as_raw(),
				);
				let handle = self.egui_ctx.load_texture(
					format!("{}:{}:{}", tile_id.x, tile_id.y, tile_id.zoom),
					color_image,
					TextureOptions::default(),
				);

				CachedTexture::Valid(Tile::Raster(handle))
			}).clone()
	}
}

impl Tiles for FractalTiles {
	fn at(&mut self, tile_id: TileId) -> Option<TilePiece> {
		(0..=tile_id.zoom).rev().find_map(|zoom_candidate| {
			let (donor_tile_id, uv) = interpolate_from_lower_zoom(tile_id, zoom_candidate);
			match self.load(donor_tile_id) {
				CachedTexture::Valid(texture) => Some(TilePiece::new(texture.clone(), uv)),
				CachedTexture::Invalid => {
					trace!("tile invalid");
					None
				},
			}
		})
	}

	fn attribution(&self) -> Attribution {
		Attribution {
			text: "Mandelbrot Set",
			url: "",
			logo_dark: None,
			logo_light: None
		}
	}

	fn tile_size(&self) -> u32 {
		256
	}
}

// https://docs.rs/walkers/latest/src/walkers/tiles.rs.html#102-106
pub(crate) fn interpolate_from_lower_zoom(tile_id: TileId, available_zoom: u8) -> (TileId, Rect) {
	assert!(tile_id.zoom >= available_zoom);

	let dzoom = 2u32.pow((tile_id.zoom - available_zoom) as u32);

	let x = (tile_id.x / dzoom, tile_id.x % dzoom);
	let y = (tile_id.y / dzoom, tile_id.y % dzoom);

	let zoomed_tile_id = TileId {
		x: x.0,
		y: y.0,
		zoom: available_zoom,
	};

	let z = (dzoom as f32).recip();

	let uv = Rect::from_min_max(
		pos2(x.1 as f32 * z, y.1 as f32 * z),
		pos2(x.1 as f32 * z + z, y.1 as f32 * z + z),
	);

	(zoomed_tile_id, uv)
}