#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use std::sync::Arc;
use eframe::egui;
use egui::{ColorImage, Context, Image, TextureHandle, TextureOptions};
use egui::epaint::ImageDelta;
use image::{ImageBuffer, Rgba};
use rayon::iter::Fold;
use crate::cpu;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

pub fn run() -> eframe::Result {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
		..Default::default()
	};
	eframe::run_native(
		"My egui App",
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
	state: State
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
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			if self.state.old_from.is_none() { // it's enough to check one field (`to`) in this case 'cause they're like paired
				self.render(ctx);
			}

			egui::CentralPanel::default().show(ctx, |ui| {
				if let Some(tex) = &self.texture {
					ui.image(tex);
				}
			});
		});
	}
}