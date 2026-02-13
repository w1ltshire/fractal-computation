use std::collections::HashMap;
use std::time::Duration;
use walkers::TileId;

#[cfg(target_arch = "wasm32")]
use wasm_thread as threads;
#[cfg(not(target_arch = "wasm32"))]
use std::thread as threads;

use eframe::epaint::ColorImage;
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
	Completed(TileId, ColorImage),
	/// Thread has not yet completed its work
	NotReady(TileId),
}

pub fn create_parent_thread() -> (Sender<ThreadMessage>, Receiver<ThreadMessage>) {
	let (sender_to_parent, receiver_from_main) = kanal::unbounded::<ThreadMessage>();
	let (sender_to_main, receiver_from_parent) = kanal::unbounded::<ThreadMessage>();

	threads::spawn(move || {
		debug!("parent thread is up");
		let mut receivers: HashMap<TileId, (Sender<ThreadMessage>, Receiver<ThreadMessage>)> = HashMap::new();

		loop {
			match receiver_from_main.try_recv() {
				Ok(Some(msg)) => {
					match msg {
						ThreadMessage::CreateWork(tile_id, iters) => {
							receivers.insert(tile_id, worker_thread(tile_id, iters));
						}
						ThreadMessage::Poll(tile_id) => {
							if let Some((sender_to_worker, receiver_from_worker)) = receivers.get_mut(&tile_id) {
								if let Err(e) = sender_to_worker.send(ThreadMessage::Poll(tile_id)) {
									debug!("failed to send Poll to worker: {e:?}");
								}

								match receiver_from_worker.try_recv() {
									Ok(Some(msg)) => {
										debug!("thread received message from worker {tile_id:?}: {msg:?}");
										match msg {
											ThreadMessage::Completed(tile_id, color_image) => {
												debug!("thread for {tile_id:?} says it has completed its task");
												let _ = sender_to_main.send(ThreadMessage::Completed(tile_id, color_image));
												receivers.remove(&tile_id);
											}
											ThreadMessage::NotReady(tile_id) => {
												debug!("thread for {tile_id:?} says it has not completed its task");
												let _ = sender_to_main.send(ThreadMessage::NotReady(tile_id));
											}
											_ => {}
										}
									}
									Ok(None) => {
										let _ = sender_to_main.send(ThreadMessage::NotReady(tile_id));
									}
									Err(e) => {
										debug!("error receiving from worker: {e:?}");
										let _ = sender_to_main.send(ThreadMessage::NotReady(tile_id));
									}
								}
							} else {
								let _ = sender_to_main.send(ThreadMessage::NotReady(tile_id));
							}
						}
						_ => {}
					}
				}
				Ok(None) => {
					std::thread::sleep(Duration::from_millis(20));
				}
				Err(e) => {
					debug!("parent receiver error: {e:?}");
					break;
				}
			}
		}
	});

	(sender_to_parent, receiver_from_parent)
}

pub fn worker_thread(tile_id: TileId, iterations: usize) -> (Sender<ThreadMessage>, Receiver<ThreadMessage>) {
	let (sender_to_worker, receiver_from_parent) = kanal::unbounded::<ThreadMessage>();
	let (sender_to_parent, receiver_from_worker) = kanal::unbounded::<ThreadMessage>();

	threads::spawn(move || {
		debug!("worker thread for tile {tile_id:?} is up");
		let d = 1u32 << (tile_id.zoom as u32);
		let scale = 3.0 / (d as f64);
		let x_center = (tile_id.x as f64) * scale - 2.0;
		let y_center = (tile_id.y as f64) * scale - 1.5;

		let from = (x_center, y_center);
		let to = (x_center + scale, y_center + scale);

		let samples = (512, 512);
		let max_iter = iterations;

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

		debug!("worker for {tile_id:?} has finished computation, waiting for poll...");

		loop {
			match receiver_from_parent.try_recv() {
				Ok(Some(msg)) => match msg {
					ThreadMessage::Poll(_) => {
						let _ = sender_to_parent.send(ThreadMessage::Completed(tile_id, color_image.clone()));
						break;
					}
					_ => {}
				},
				Ok(None) => {
					std::thread::sleep(Duration::from_millis(20));
				}
				Err(e) => {
					debug!("worker receiver error: {e:?}");
					let _ = sender_to_parent.send(ThreadMessage::NotReady(tile_id));
					break;
				}
			}
		}
	});

	(sender_to_worker, receiver_from_worker)
}
