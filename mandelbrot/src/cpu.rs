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
	real: std::ops::Range<f64>,
	complex: std::ops::Range<f64>,
	samples: (usize, usize),
	max_iter: usize,
	exp: f64,
) -> Vec<(f64, f64, usize)> {
	let (nx, ny) = samples;
	let dx = (real.end - real.start) / nx as f64;
	let dy = (complex.end - complex.start) / ny as f64;
	let mut out = Vec::with_capacity(nx * ny);

	let bailout_sq = 1e10f64;

	if (exp - 2.0).abs() < f64::EPSILON {
		for j in 0..ny {
			let cy = complex.start + j as f64 * dy;
			for i in 0..nx {
				let cx = real.start + i as f64 * dx;
				let mut zx = 0.0f64;
				let mut zy = 0.0f64;
				let mut iter = 0usize;
				while iter < max_iter {
					let zx2 = zx * zx;
					let zy2 = zy * zy;
					if zx2 + zy2 > bailout_sq {
						break;
					}
					let new_zx = zx2 - zy2 + cx;
					let new_zy = 2.0 * zx * zy + cy;
					zx = new_zx;
					zy = new_zy;
					iter += 1;
				}
				out.push((cx, cy, iter));
			}
		}
		return out;
	}

	for j in 0..ny {
		let cy = complex.start + j as f64 * dy;
		for i in 0..nx {
			let cx = real.start + i as f64 * dx;
			let mut zx = 0.0f64;
			let mut zy = 0.0f64;
			let mut iter = 0usize;
			while iter < max_iter {
				let zx2 = zx * zx;
				let zy2 = zy * zy;
				let r2 = zx2 + zy2;
				if r2 > bailout_sq {
					break;
				}
				let r = r2.sqrt();
				let theta = zy.atan2(zx);
				let r_pow = r.powf(exp);
				let new_theta = theta * exp;
				let new_zx = r_pow * new_theta.cos() + cx;
				let new_zy = r_pow * new_theta.sin() + cy;
				zx = new_zx;
				zy = new_zy;
				iter += 1;
			}
			out.push((cx, cy, iter));
		}
	}

	out
}

