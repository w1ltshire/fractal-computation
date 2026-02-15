use egui::{DragValue, RichText};
use walkers::{Map, MapMemory};
use crate::tiles::FractalTiles;

pub struct App {
	tiles: FractalTiles,
	map_memory: MapMemory,
}

impl App {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let mut map_memory = MapMemory::default();
		map_memory.set_zoom(1.5).unwrap();
		Self {
			tiles: FractalTiles::new(cc.egui_ctx.clone()),
			map_memory,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();
		if self.tiles.mandelbrot_set_properties != self.tiles.old_mandelbrot_set_properties {

		}

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::MenuBar::new().ui(ui, |ui| {
				let is_web = cfg!(target_arch = "wasm32");
				if !is_web {
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
					});
					ui.add_space(32.0);
				}

				egui::widgets::global_theme_preference_buttons(ui);
			});
		});

		egui::SidePanel::right("side_panel").exact_width(180.0).show(ctx, |ui| {
			ui.label(RichText::new("Parameters").size(18.0));
			ui.separator();
			render_settings(ui, self);
			ui.separator();
			position_settings(ui, self);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.add(
				Map::new(
					Some(&mut self.tiles),
					&mut self.map_memory,
					walkers::lat_lon(0.0, 0.0)
				)
			);
		});
	}
}

fn render_settings(ui: &mut egui::Ui, app: &mut App) {
	ui.label(RichText::new("Render settings").size(14.0));
	ui.horizontal(|ui| {
		ui.label("Iterations: ")
			.on_hover_text(RichText::new("Level of detail"));
		ui.add(DragValue::new(&mut app.tiles.mandelbrot_set_properties.iterations));
	});
	ui.horizontal(|ui| {
		ui.label("Exponent: ")
			.on_hover_text(RichText::new("Level of symmetry"));
		ui.add(DragValue::new(&mut app.tiles.mandelbrot_set_properties.exponent));
	});
	ui.horizontal(|ui| {
		ui.label("Samples: ");
		ui.add(DragValue::new(&mut app.tiles.mandelbrot_set_properties.samples.0));
		ui.add(DragValue::new(&mut app.tiles.mandelbrot_set_properties.samples.1));
	});
}

fn position_settings(ui: &mut egui::Ui, app: &mut App) {
	ui.label(RichText::new("Position").size(14.0));
	ui.horizontal(|ui| {
		ui.label("Zoom: ");
		ui.add(DragValue::new(&mut app.map_memory.zoom.0));
	});
}