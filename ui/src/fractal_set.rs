use egui::ColorImage;
use image::{ImageBuffer, Rgba};
use walkers::TileId;
use crate::tiles::MandelbrotSetProperties;

pub fn generate_color_image(tile_id: TileId, props: MandelbrotSetProperties) -> ColorImage {
	let d = 1u32 << (tile_id.zoom as u32);
	let scale = 3.0 / (d as f64);
	let x_center = (tile_id.x as f64) * scale - 2.0;
	let y_center = (tile_id.y as f64) * scale - 1.5;

	let from = (x_center, y_center);
	let to = (x_center + scale, y_center + scale);


	let mut img = ImageBuffer::<Rgba<u8>, _>::new(256, 256);

	for (c_re, c_im, count) in mandelbrot::cpu::mandelbrot_set(
		from.0..to.0,
		from.1..to.1,
		props.samples,
		props.iterations,
		props.exponent as f64
	) {
		let x = ((c_re - from.0) / scale * 256.0) as u32;
		let y = ((c_im - from.1) / scale * 256.0) as u32;

		let color = if count < props.iterations {
			let intensity = (count as u8) % 255;
			Rgba([intensity, intensity, intensity, 255])
		} else {
			Rgba([0, 0, 0, 255])
		};

		if x < 256 && y < 256 {
			img.put_pixel(x, y, color);
		}
	}

	ColorImage::from_rgba_unmultiplied(
		[256usize, 256usize],
		img.as_raw(),
	)
}