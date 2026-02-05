use itertools::Itertools;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::simd::prelude::*;

pub fn mandelbrot(data: &mut [[u8; 4]], from: (f64, f64), to: (f64, f64), width: u32, height: u32, zoom: f64, center: (f64, f64)) {
	let height = height as usize;
	let width = width as usize;

	let data = Arc::new(Mutex::new(data));

	let center_x = center.0;
	let center_y = center.1;

	let new_width = (to.0 - from.0) / zoom;
	let new_height = (to.1 - from.1) / zoom;

	let new_from = (
		center_x - new_width / 2.0,
		center_y - new_height / 2.0,
	);

	let new_to = (
		center_x + new_width / 2.0,
		center_y + new_height / 2.0,
	);

	(0..height).into_par_iter().for_each(|x| {
		let cy_value = new_from.1 + (new_to.1 - new_from.1) * x as f64 / height as f64;

		for chunk in &(0..width).chunks(4) {
			let mut iter_counts = [0usize; 4];
			let mut x_sqr = f64x4::splat(0.0);
			let mut y_sqr = f64x4::splat(0.0);
			let mut zx = f64x4::splat(0.0);
			let mut zy = f64x4::splat(0.0);
			let mut current_iterations = [0usize; 4];

			let mut active = f64x4::splat(1.0);

			for (i, y) in chunk.into_iter().enumerate() {
				let cx = new_from.0 + (new_to.0 - new_from.0) * y as f64 / width as f64;

				zx[i] = 0.0;
				zy[i] = 0.0;

				while active[i] != 0.0 && current_iterations[i] < 255 {
					let temp = zx[i] * zy[i];
					zy[i] = (temp * 2.0) + cy_value;
					zx[i] = x_sqr[i] - y_sqr[i] + cx;

					x_sqr[i] = zx[i] * zx[i];
					y_sqr[i] = zy[i] * zy[i];

					if x_sqr[i] + y_sqr[i] >= 4.0 {
						active[i] = 0.0;
					} else {
						current_iterations[i] += 1;
					}
				}

				iter_counts[i] = current_iterations[i];
				let index = x * width;
				let mut data_ref = data.lock().unwrap();
				if i < y {
					let iter = iter_counts[i];
					let pixel_index = index + y;
					data_ref[pixel_index] = [255 - iter as u8, 255 - iter as u8, 255 - iter as u8, 255];
				}
			}
		}
	});
}