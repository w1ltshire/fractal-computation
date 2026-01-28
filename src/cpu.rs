use crate::Complex;

pub fn mandelbrot_color(c: &Complex, upper: u32) -> u32 {
	let mut z = Complex::ZERO;
	for i in 0..=upper {
		z = z * z + c;
		if z.norm_sqr() > 4.0 {
			return i
		}
	}
	upper
}

pub fn mandelbrot(data: &mut [[u8; 4]], from: &Complex, to: &Complex, width: u32, height: u32) {
	for y in 0..height {
		for x in 0..width {
			let c = Complex::new(
				from.re + (to.re - from.re) * x as f64 / width as f64,
				from.im + (to.im - from.im) * y as f64 / height as f64,
			);
			let iter = mandelbrot_color(&c, 255);
			let index = (y * width + x) as usize;
			data[index] = [ 255 - iter as u8, 255 - iter as u8, 255 - iter as u8, 255];
		}
	}
}