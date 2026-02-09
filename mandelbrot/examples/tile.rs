use mandelbrot::cpu::MandelbrotTile;

fn main() {
	let mut tile = MandelbrotTile {
		data: vec![[0u8; 4]; 40],
		width: 400,
		height: 400,
		x: 0.0,
		y: 0.0,
		iters: 100,
	};
	//mandelbrot::cpu::mandelbrot_simd_tiled(&mut tile);
	println!("{:?}", tile);
}