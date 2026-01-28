use macroquad::miniquad::window;
use macroquad::prelude::{draw_texture_ex, next_frame, DrawTextureParams, Image, Texture2D, Vec2, BLACK, WHITE};

type Complex = num_complex::Complex<f64>;

mod cpu;

#[macroquad::main("Mandelbrot")]
async fn main() {
    let (width, height) = window::screen_size();

    let from = Complex::new(-1.7, -1.3);
    let to = Complex::new(1.0, 1.3);

    let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);
    let bmp = Texture2D::from_image(&image);
    cpu::mandelbrot(image.get_image_data_mut(), &from, &to, width as u32, height as u32);
    bmp.update(&image);
    loop {
        let params = DrawTextureParams {
            dest_size: Some(Vec2::new(width, height)),
            ..Default::default()
        };
        draw_texture_ex(&bmp, 0.0, 0.0, WHITE, params);

        next_frame().await
    }
}