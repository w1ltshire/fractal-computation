use walkers::TileId;

#[cfg(target_arch = "wasm32")]
use wasm_thread as threads;
#[cfg(not(target_arch = "wasm32"))]
use std::thread as threads;
use egui::ColorImage;
use kanal::Sender;
use log::debug;

/// Possible messages to be passed/received to/from threads (parent and workers)
#[derive(Debug, PartialEq, Clone)]
pub enum ThreadMessage {
	CreateWork(TileId),
	Poll(TileId),
	Completed(ColorImage)
}

pub fn create_parent_thread() -> Sender<ThreadMessage> {
	let (sender, receiver) = kanal::unbounded::<ThreadMessage>();
	threads::spawn(move || {
		debug!("parent thread is up");
		while let Ok(msg) = receiver.try_recv() {
			if msg.is_some() {
				debug!("{:?}", msg.unwrap());
			}
		}
	});
	sender
}