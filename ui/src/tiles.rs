use eframe::emath::{pos2, Rect};
use eframe::epaint::ColorImage;
use eframe::epaint::textures::TextureOptions;
use egui::Context;
use image::{ImageBuffer, Rgba};
use kanal::Sender;
use log::debug;
use lru::LruCache;
use walkers::{Tile, TileId, TilePiece, Tiles};
use walkers::sources::Attribution;
use crate::threads::ThreadMessage;

pub type Set = Vec<(f64, f64, usize)>;

#[derive(Clone)]
enum CachedTexture {
	Valid(Tile),
	#[allow(unused)]
	Invalid,
	#[allow(unused)]
	Pending
}

pub struct FractalTiles {
	tiles: LruCache<TileId, CachedTexture>,
	egui_ctx: Context,
	pub(crate) mandelbrot_set_properties: MandelbrotSetProperties,
	parent_thread_sender: Sender<ThreadMessage>,
}

#[derive(Default)]
pub struct MandelbrotSetProperties {
	pub iterations: usize,
}

impl FractalTiles {
	pub fn new(egui_ctx: Context, parent_thread_sender: Sender<ThreadMessage>) -> Self {
		Self {
			tiles: LruCache::unbounded(),
			egui_ctx,
			mandelbrot_set_properties: MandelbrotSetProperties {
				iterations: 255,
			},
			parent_thread_sender,
		}
	}

	fn load(&mut self, tile_id: TileId) -> CachedTexture {
		self.tiles
			.get_or_insert(tile_id, || {
				self.parent_thread_sender.send(ThreadMessage::CreateWork(tile_id, self.mandelbrot_set_properties.iterations)).unwrap();
				let scale = 3.0 / (1 << tile_id.zoom) as f64;
				let x_center = (tile_id.x as f64) * scale - 2.0;
				let y_center = (tile_id.y as f64) * scale - 1.5;

				let from = (x_center, y_center);
				let to = (x_center + scale, y_center + scale);

				let real_range = from.0..to.0;
				let complex_range = from.1..to.1;
				let samples = (512, 512);
				let max_iter = self.mandelbrot_set_properties.iterations;

				let mut img = ImageBuffer::<Rgba<u8>, _>::new(256, 256);

				for (c_re, c_im, count) in mandelbrot::cpu::mandelbrot_set(from.0..to.0, from.1..to.1, samples, max_iter) {
					let x = ((c_re - from.0) / scale * 256.0) as u32;
					let y = ((c_im - from.1) / scale * 256.0) as u32;

					let color = if count < max_iter {
						let intensity = (count as u8) % 255;
						Rgba([intensity, intensity, intensity, 255])
					} else {
						Rgba([0, 0, 0, 255])
					};

					if x < 256 && y < 256 {
						img.put_pixel(x, y, color);
					}
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
				CachedTexture::Invalid | CachedTexture::Pending => {
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