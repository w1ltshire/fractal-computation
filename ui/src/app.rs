use kanal::Sender;
use walkers::{Map, MapMemory};
use crate::threads;
use crate::threads::ThreadMessage;
use crate::tiles::FractalTiles;

pub struct App {
	tiles: FractalTiles,
	map_memory: MapMemory,
	parent_thread_sender: Sender<ThreadMessage>
}

impl App {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let mut map_memory = MapMemory::default();
		let (parent_thread_sender, parent_thread_receiver) = threads::create_parent_thread();
		map_memory.set_zoom(1.).unwrap();
		Self {
			tiles: FractalTiles::new(cc.egui_ctx.clone(), parent_thread_sender.clone()),
			map_memory,
			parent_thread_sender
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::MenuBar::new().ui(ui, |ui| {
				let is_web = cfg!(target_arch = "wasm32");
				if !is_web {
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
					});
					ui.add_space(16.0);
				}

				egui::widgets::global_theme_preference_buttons(ui);
			});
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