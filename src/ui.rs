#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use egui::{ColorImage, Context, TextureHandle, TextureOptions, Ui};
use egui::epaint::ImageDelta;
use image::{ImageBuffer, Rgba};
use crate::cpu;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

pub fn run() -> eframe::Result {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
		..Default::default()
	};
	eframe::run_native(
		"fractal-computation",
		options,
		Box::new(|cc| {
			egui_extras::install_image_loaders(&cc.egui_ctx);

			Ok(Box::<App>::default())
		}),
	)
}

struct App {
	texture: Option<TextureHandle>,
	data: Vec<[u8; 4]>,
	state: State,
	zoom: f64,
	zoom_center: (f64, f64),
	old_zoom: f64,
	old_zoom_center: (f64, f64),
}

#[derive(Default)]
struct State {
	from: (f64, f64),
	to: (f64, f64),
	old_from: Option<(f64, f64)>,
	old_to: Option<(f64, f64)>,
}

impl App {
	fn render(&mut self, ctx: &Context) {
		cpu::mandelbrot(
			&mut self.data,
			self.state.from,
			self.state.to,
			WIDTH,
			HEIGHT,
			self.zoom,
			self.zoom_center,
		);

		self.state.old_from = Some(self.state.from);
		self.state.old_to = Some(self.state.to);

		let mut img = ImageBuffer::<Rgba<u8>, _>::new(WIDTH, HEIGHT);
		for (i, pixel) in self.data.iter().enumerate() {
			let x = (i as u32) % WIDTH;
			let y = (i as u32) / WIDTH;
			img.put_pixel(x, y, Rgba(*pixel));
		}
		let color_image = ColorImage::from_rgba_unmultiplied(
			[WIDTH as usize, HEIGHT as usize],
			img.as_raw(),
		);

		let tex = match &self.texture {
			Some(t) => {
				ctx.tex_manager().write().set(
					t.id(),
					ImageDelta::full(color_image, TextureOptions::default()),
				);
				t.clone()
			}
			None => {
				ctx.load_texture(
					"mandelbrot",
					color_image,
					TextureOptions::default(),
				)
			}
		};
		self.texture = Some(tex);
	}
}

impl Default for App {
	fn default() -> Self {
		let data = vec![[0; 4]; (WIDTH * HEIGHT) as usize];
		Self {
			texture: None,
			data,
			state: State {
				from: (-2.0, -1.5),
				to: (1.0, 1.5),
				old_from: None,
				old_to: None,
			},
			zoom: 1.0,
			zoom_center: (0.0, 0.0),
			old_zoom: 1.0,
			old_zoom_center: (0.0, 0.0),
		}
	}
}

fn ui_image_responsive(ui: &mut Ui, texture: &TextureHandle) {
	let available = ui.available_rect_before_wrap();

	let original_size = texture.size();
	let aspect = original_size[0] as f32 / original_size[1] as f32;

	let width = available.width();
	let height = (width / aspect).min(available.height());

	ui.allocate_ui_with_layout(
		egui::vec2(width, height),
		egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
		|ui| {
			ui.image(texture);
		},
	);
}

impl eframe::App for App {
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			if let Some(old_from) = self.state.old_from && let Some(old_to) = self.state.old_to {
				if old_to != self.state.to {
					self.render(ctx);
				}
				if old_from != self.state.from {
					self.render(ctx);
				}
			} else {
				self.render(ctx);
			}
			if self.old_zoom != self.zoom || self.old_zoom_center != self.zoom_center {
				self.render(ctx);
			}
			ui.add(egui::Slider::new(&mut self.zoom, 1.0..=100.0).text("zoom"));
			ui.add(egui::Slider::new(&mut self.zoom_center.0, -10.0..=10.0).text("zoom_center.0"));
			ui.add(egui::Slider::new(&mut self.zoom_center.1, -10.0..=10.0).text("zoom_center.1"));

			if let Some(tex) = &self.texture {
				ui_image_responsive(ui, tex);
			}
		});
	}
}
