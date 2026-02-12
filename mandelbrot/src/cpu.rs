use itertools::Itertools;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::simd::prelude::*;

pub fn mandelbrot_simd(data: &mut [[u8; 4]], from: (f64, f64), to: (f64, f64), width: u32, height: u32, iters: usize) {
	let width = width as usize;
	let height = height as usize;

	let data = Arc::new(Mutex::new(data));

	(0..height).into_par_iter().for_each(|x| {
		let cy_value = from.1 + (to.1 - from.1) * (x as f64 / height as f64);
		let mut local_data = vec![[0u8; 4]; width];

		for chunk in &(0..width).chunks(4) {
			let mut zx = f64x4::splat(0.0);
			let mut zy = f64x4::splat(0.0);
			let mut current_iterations = [0usize; 4];
			let mut active = f64x4::splat(1.0);

			for (i, y) in chunk.into_iter().enumerate() {
				let cx = from.0 + (to.0 - from.0) * (y as f64 / width as f64);
				zx[i] = 0.0;
				zy[i] = 0.0;

				while active[i] != 0.0 && current_iterations[i] < iters {
					let x_sqr = zx[i] * zx[i];
					let y_sqr = zy[i] * zy[i];

					if x_sqr + y_sqr >= 4.0 {
						active[i] = 0.0;
					} else {
						zy[i] = (zx[i] * zy[i] * 2.0) + cy_value;
						zx[i] = x_sqr - y_sqr + cx;
						current_iterations[i] += 1;
					}
				}
				local_data[y] = [255 - current_iterations[i] as u8, 255 - current_iterations[i] as u8, 255 - current_iterations[i] as u8, 255];
			}
		}

		let mut data_ref = data.lock().unwrap();
		let index = x * width;
		for y in 0..width {
			data_ref[index + y] = local_data[y];
		}
	});
}

pub fn mandelbrot_set(
	real: Range<f64>,
	complex: Range<f64>,
	samples: (usize, usize),
	max_iter: usize
) -> Vec<(f64, f64, usize)> {
	let step = (
		(real.end - real.start) / samples.0 as f64,
		(complex.end - complex.start) / samples.1 as f64,
	);

	(0..(samples.0 * samples.1))
		.into_par_iter()
		.map(|k| {
			let c = (
				real.start + step.0 * (k % samples.0) as f64,
				complex.start + step.1 * (k / samples.0) as f64,
			);
			let mut z = (0.0, 0.0);
			let mut cnt = 0;
			while cnt < max_iter && z.0 * z.0 + z.1 * z.1 <= 1e10 {
				z = (z.0 * z.0 - z.1 * z.1 + c.0, 2.0 * z.0 * z.1 + c.1);
				cnt += 1;
			}
			(c.0, c.1, cnt)
		})
		.collect()
}