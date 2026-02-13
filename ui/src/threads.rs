use std::collections::HashMap;
use walkers::TileId;

#[cfg(target_arch = "wasm32")]
use wasm_thread as threads;
#[cfg(not(target_arch = "wasm32"))]
use std::thread as threads;
use egui::ColorImage;
use image::{ImageBuffer, Rgba};
use kanal::{Receiver, Sender};
use log::debug;

/// Possible messages to be passed/received to/from threads (parent and workers)
#[derive(Debug, PartialEq, Clone)]
pub enum ThreadMessage {
	/// Make the parent thread spawn a worker computing the fractal set
	CreateWork(TileId, usize),
	/// Poll the worker thread assigned to this tile id
	Poll(TileId),
	/// The ready, calculated fractal set image
	Completed(ColorImage),
	/// Thread has not yet completed its work
	NotReady,
}

/// Create a parent thread
pub fn create_parent_thread() -> (Sender<ThreadMessage>, Receiver<ThreadMessage>) {
	let (sender_to_parent, receiver_from_main) = kanal::unbounded::<ThreadMessage>();
	let (sender_to_main, receiver_from_parent) = kanal::unbounded::<ThreadMessage>();

	threads::spawn(move || {
		debug!("parent thread is up");
		let mut receivers: HashMap<TileId, (Sender<ThreadMessage>, Receiver<ThreadMessage>)> = HashMap::new();
		while let Ok(msg) = receiver_from_main.try_recv() {
			match msg {
				Some(ThreadMessage::CreateWork(tile_id, iters)) => {
					receivers.insert(tile_id, worker_thread(tile_id, iters));
				}
				Some(ThreadMessage::Poll(tile_id)) => {
					let pair = receivers.get_mut(&tile_id).unwrap();
					if let Ok(Some(msg)) = pair.1.try_recv() {
						debug!("thread received message from worker {tile_id:?}: {msg:?}");
						match msg {
							ThreadMessage::Completed(color_image) => {
								sender_to_main.send(ThreadMessage::Completed(color_image)).unwrap();
							}
							ThreadMessage::NotReady => {
								sender_to_main.send(ThreadMessage::NotReady).unwrap();
							}
							_ => {} // shouldn't receive anything else from the worker, ignore all other patterns
						}
					} else {

					}
				}
				_ => {}
			}
		}
	});
	(sender_to_parent, receiver_from_parent)
}

pub fn worker_thread(tile_id: TileId, iterations: usize) -> (Sender<ThreadMessage>, Receiver<ThreadMessage>) {
	// one pair of sender/receiver for communication to the worker!
	let (sender_to_worker, receiver_from_parent) = kanal::unbounded::<ThreadMessage>();
	let (sender_to_parent, receiver_from_worker) = kanal::unbounded::<ThreadMessage>();

	threads::spawn(move || {
		debug!("worker thread for tile {tile_id:?} is up");
		let scale = 3.0 / (1 << tile_id.zoom) as f64;
		let x_center = (tile_id.x as f64) * scale - 2.0;
		let y_center = (tile_id.y as f64) * scale - 1.5;

		let from = (x_center, y_center);
		let to = (x_center + scale, y_center + scale);

		let samples = (512, 512);

		let mut img = ImageBuffer::<Rgba<u8>, _>::new(256, 256);

		for (c_re, c_im, count) in mandelbrot::cpu::mandelbrot_set(from.0..to.0, from.1..to.1, samples, iterations) {
			let x = ((c_re - from.0) / scale * 256.0) as u32;
			let y = ((c_im - from.1) / scale * 256.0) as u32;

			let color = if count < iterations {
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

		// if we receive a poll request from the parent thread then we'll check if color image is ready,
		// and then send ThreadMessage::Completed(ColorImage) to it (otherwise send ThreadMessage:NotReady but idk how can this happen),
		// so to do this we: have a receiver from the parent and a sender to the parent

		while let Ok(Some(msg)) = receiver_from_parent.try_recv() {
			match msg {
				ThreadMessage::Poll(_) => {
					sender_to_parent.send(ThreadMessage::Completed(color_image.clone())).unwrap();
				},
				_ => {}
			}
		};
	});
	(sender_to_worker, receiver_from_worker)
}
