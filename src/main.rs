use image::{ImageBuffer, Rgba};

type Complex = num_complex::Complex<f64>;

mod cpu;
mod gpu;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    const WIDTH: u32 = 3840;
    const HEIGHT: u32 = 2160;

    let from = Complex::new(-2.0, -1.5);
    let to = Complex::new(1.0, 1.5);

    let mut data: Vec<[u8; 4]> = vec![[0; 4]; (WIDTH * HEIGHT) as usize];

    cpu::mandelbrot(&mut data, &from, &to, WIDTH, HEIGHT);

    let mut imgbuf = ImageBuffer::new(WIDTH, HEIGHT);

    for (i, pixel) in data.into_iter().enumerate() {
        let x = (i as u32) % WIDTH;
        let y = (i as u32) / WIDTH;
        imgbuf.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]));
    }

    imgbuf.save("mandelbrot.png").unwrap();
    Ok(())
}