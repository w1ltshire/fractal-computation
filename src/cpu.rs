use std::sync::{Arc, Mutex};
use crate::Complex;
use rayon::prelude::*;

fn mandelbrot_color(cx: f64, cy: f64, max_iterations: usize) -> usize {
	let mut zx = 0.0; // re
	let mut zy = 0.0; // im
	let mut x_sqr = 0.0;
	let mut y_sqr = 0.0;
	let mut i = 0;

	while x_sqr + y_sqr < 4.0 && i < max_iterations {
		zy = 2.0 * zy * zx + cy;
		zx = x_sqr - y_sqr + cx;
		x_sqr = zx * zx;
		y_sqr = zy * zy;
		i += 1;
	}

	i
}

pub fn mandelbrot(data: &mut [[u8; 4]], from: &Complex, to: &Complex, width: u32, height: u32) {
	let height = height as usize;
	let width = width as usize;

	let data = Arc::new(Mutex::new(data)); // i think this may slow down the computation a lil bit?
	// but yeah it's still faster than single-threaded

	(0..height).into_par_iter().for_each(|x| {
		for y in 0..width {
			let c = Complex::new(
				from.re + (to.re - from.re) * y as f64 / width as f64,
				from.im + (to.im - from.im) * x as f64 / height as f64,
			);
			let iter = mandelbrot_color(c.re, c.im, 255);
			let index = x * width + y;

			let mut data_ref = data.lock().unwrap();
			data_ref[index] = [255 - iter as u8, 255 - iter as u8, 255 - iter as u8, 255];
		}
	});
}