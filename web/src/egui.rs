use chrono::Duration;
use wasm_bindgen::prelude::*;
use eframe::egui;
use egui::{CentralPanel, Context, DragValue, TextureHandle};
use walkers::{lon_lat, Map, MapMemory};
use crate::tiles::FractalTiles;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;

pub struct EguiApp {
	texture: Option<TextureHandle>,
	data: Vec<[u8; 4]>,
	elapsed: Duration,
	mandelbrot_set_properties: MandelbrotSetProperties,
	tiles: FractalTiles,
	map_memory: MapMemory,
}

#[derive(Default)]
struct MandelbrotSetProperties {
	iterations: usize,
}

impl EguiApp {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		Self {
			texture: None,
			data: vec![[0; 4]; (WIDTH * HEIGHT) as usize],
			elapsed: Duration::default(),
			mandelbrot_set_properties: MandelbrotSetProperties {
				iterations: 255,
			},
			tiles: FractalTiles::new(cc.egui_ctx.clone()),
			map_memory: MapMemory::default(),
		}
	}
}

impl eframe::App for EguiApp {
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		egui::SidePanel::right("my_left_panel")
			.resizable(false)
			.exact_width(250.0)
			.show(ctx, |ui| {
				ui.label(egui::RichText::new("Parameters").size(20.0));
				ui.separator();
				ui.label(egui::RichText::new("Render settings").size(15.0));
				ui.horizontal(|ui| {
					ui.label("Iterations:");
					ui.add(DragValue::new(&mut self.mandelbrot_set_properties.iterations));
				});
				ui.separator();
				if ui.add_sized([240.0, 20.0], egui::Button::new("render")).clicked() {
					//self.render(ctx);
				}
				ui.label(format!("rendering elapsed {:?} s", self.elapsed.as_seconds_f64()));
			});
		CentralPanel::default().show(ctx, |ui| {
			ui.add(Map::new(
				Some(&mut self.tiles),
				&mut self.map_memory,
				lon_lat(0.0, 0.0)
			));
		});
	}
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
	runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
	#[expect(clippy::new_without_default)]
	#[wasm_bindgen(constructor)]
	pub fn new() -> Self {

		Self {
			runner: eframe::WebRunner::new(),
		}
	}

	#[wasm_bindgen]
	pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) {
		self.runner
			.start(
				canvas,
				eframe::WebOptions::default(),
				Box::new(|cc| Ok(Box::new(EguiApp::new(cc))),)
			)
			.await.unwrap()
	}
}