use image::{ImageBuffer, Rgb};
use image::imageops::invert;
use std::path::Path;


/// Computes the escape time for a point in the Mandelbrot set.
///
/// `c` is the complex number for the point, and `max_iter` is the maximum
/// number of iterations to compute. Returns the escape time as a floating
/// point number.
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

/// Renders the Mandelbrot set as an image.
///
/// `width` and `height` are the dimensions of the image, and `max_iter` is
/// the maximum number of iterations to compute for each point.
fn render_mandelbrot(width: u32, height: u32, max_iter: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let scalex: f64 = 3.0 / width as f64;
    let scaley: f64 = 3.0 / height as f64;

    ImageBuffer::from_fn(width, height, |x: u32, y: u32| {
        let c = (x as f64 * scalex - 2.0, y as f64 * scaley - 1.5);
        let mut n_iterations: f64 = mandelbrot(c, max_iter);

        n_iterations /= max_iter as f64;

        let (r, g, b): (u8, u8, u8) = color_gradient(n_iterations);
        image::Rgb([r, g, b])
    })
}

/// Maps a number between 0 and 1 to a color gradient.
///
/// `i` is the number to map. Returns an RGB color as a tuple of three bytes.
fn color_gradient(i: f64) -> (u8, u8, u8) {
    let r: u8 = (255.0 * i) as u8;
    let g: u8 = (255.0 * (1.0 - i)) as u8;
    let b: u8 = (255.0 * (1.0 - i.abs())) as u8;
    (r, g, b)
}

fn main() {
    let (width, height): (u32, u32) = (800, 800);
    let max_iter: u32 = 10000;

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = render_mandelbrot(width, height, max_iter);

    invert(&mut img);

    let output_path = Path::new("mandelbrot_set.png");
    if let Err(e) = img.save(output_path) {
        eprintln!("Failed to save image: {}", e);
        std::process::exit(1);
    }

    // Display the image using the image crate
    if let Err(e) = open::that(output_path) {
        eprintln!("Failed to open image: {}", e);
        std::process::exit(1);
    }
}
