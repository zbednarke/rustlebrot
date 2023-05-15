use image::{ImageBuffer, Rgb};
use image::imageops::invert;
use std::path::Path;


fn mandelbrot(c: (f64, f64), max_iter: u32) -> f64 {
    let mut z: (f64, f64) = (0.0, 0.0);
    for i in 0..max_iter {
        let (x, y): (f64, f64) = (z.0 * z.0 - z.1 * z.1 + c.0, 2.0 * z.0 * z.1 + c.1);
        if x * x + y * y > 4.0 {
            return i as f64 - (x * x + y * y).log2().log2() / 2.0;
        }
        z = (x, y);
    }
    max_iter as f64
}

fn render_mandelbrot(width: u32, height: u32, max_iter: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let scalex: f64 = 3.0 / width as f64;
    let scaley: f64 = 3.0 / height as f64;

    ImageBuffer::from_fn(width, height, |x: u32, y: u32| {
        let cx: f64 = x as f64 * scalex - 2.0;
        let cy: f64 = y as f64 * scaley - 1.5;

        let c: (f64, f64) = (cx, cy);
        let mut i: f64 = mandelbrot(c, max_iter);

        i /= max_iter as f64;

        let (r, g, b): (u8, u8, u8) = color_gradient(i);
        image::Rgb([r, g, b])
    })
}

fn color_gradient(i: f64) -> (u8, u8, u8) {
    let r: u8 = (255.0 * i) as u8;
    let g: u8 = (255.0 * (1.0 - i)) as u8;
    let b: u8 = (255.0 * (1.0 - i.abs())) as u8;
    (r, g, b)
}

fn main() {
    let (width, height): (u32, u32) = (1600, 1600);
    let max_iter: u32 = 1000;

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = render_mandelbrot(width, height, max_iter);

    invert(&mut img);

    let output_path: &Path = Path::new("mandelbrot_set.png");
    img.save(output_path).unwrap();

    // Display the image using the image crate
    open::that("mandelbrot_set.png").unwrap();
}
