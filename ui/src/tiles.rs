use eframe::emath::{pos2, Rect};
use eframe::epaint::textures::TextureOptions;
use egui::Context;
use kanal::{Receiver as KReceiver, Sender as KSender};
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
	Pending,
}

pub struct FractalTiles {
	tiles: LruCache<TileId, CachedTexture>,
	egui_ctx: Context,
	pub(crate) mandelbrot_set_properties: MandelbrotSetProperties,
	parent_thread_sender: KSender<ThreadMessage>,
	parent_thread_receiver: KReceiver<ThreadMessage>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct MandelbrotSetProperties {
	pub iterations: usize,
	pub exponent: usize
}

impl FractalTiles {
	pub fn new(
		egui_ctx: Context,
		parent_thread_sender: KSender<ThreadMessage>,
		parent_thread_receiver: KReceiver<ThreadMessage>,
	) -> Self {
		Self {
			tiles: LruCache::unbounded(),
			egui_ctx,
			mandelbrot_set_properties: MandelbrotSetProperties { iterations: 255, exponent: 2 },
			parent_thread_sender,
			parent_thread_receiver,
		}
	}

	fn load(&mut self, tile_id: TileId) -> CachedTexture {
		if let Some(tex) = self.tiles.get(&tile_id) {
			match tex {
				CachedTexture::Valid(tile) => CachedTexture::Valid(tile.clone()),
				CachedTexture::Invalid => CachedTexture::Invalid,
				CachedTexture::Pending => {
					if let Some(piece) = self.poll_tile(tile_id) {
						self.tiles.put(tile_id, CachedTexture::Valid(piece.clone()));
						CachedTexture::Valid(piece)
					} else {
						CachedTexture::Pending
					}
				}
			}
		} else {
			self.tiles.get_or_insert(tile_id, || {
				let _ = self.parent_thread_sender.send(ThreadMessage::CreateWork(tile_id, self.mandelbrot_set_properties));
				CachedTexture::Pending
			}).clone()
		}
	}

	fn poll_tile(&mut self, tile_id: TileId) -> Option<Tile> {
		let _ = self.parent_thread_sender.send(ThreadMessage::Poll(tile_id));
		loop {
			match self.parent_thread_receiver.try_recv() {
				Ok(Some(msg)) => match msg {
					ThreadMessage::Completed(tid, color_image) => {
						if tid == tile_id {
							debug!("match..");
							let handle = self.egui_ctx.load_texture(
								format!("{}:{}:{}", tile_id.x, tile_id.y, tile_id.zoom),
								color_image,
								TextureOptions::default(),
							);
							return Some(Tile::Raster(handle));
						} else {
							debug!("mismatch..");
							self.tiles.put(tid, CachedTexture::Valid(Tile::Raster(
								self.egui_ctx.load_texture(
									format!("{}:{}:{}", tid.x, tid.y, tid.zoom),
									color_image,
									TextureOptions::default(),
								)
							)));
						}
					}
					ThreadMessage::NotReady(tid) => {
						if tid == tile_id {
							return None;
						} else {
							continue;
						}
					}
					_ => {}
				},
				Ok(None) => return None,
				Err(_) => return None,
			}
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