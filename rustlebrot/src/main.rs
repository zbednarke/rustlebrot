use image::{ImageBuffer, Rgb};
use image::imageops::invert;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use rayon::prelude::*;
use colorgrad::sinebow;

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
            return i as f64;
        }
        z = (x, y);
    }
    max_iter as f64
}

fn render_mandelbrot(width: u32, height: u32, max_iter: u32, x_range: (f64, f64), y_range: (f64, f64)) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let scalex: f64 = (x_range.1 - x_range.0) / width as f64;
    let scaley: f64 = (y_range.1 - y_range.0) / height as f64;

    let mut data = vec![0u8; (width * height * 3) as usize];

    data.par_chunks_mut(3).enumerate().for_each(|(i, chunk)| {
        let x = i as u32 % width;
        let y = i as u32 / width;

        let cx = x as f64 * scalex + x_range.0;
        let cy = y as f64 * scaley + y_range.0;

        let c = (cx, cy);
        let iter_ratio = mandelbrot(c, max_iter) / max_iter as f64;

        let (r, g, b) = color_gradient(iter_ratio);
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
    });

    ImageBuffer::from_vec(width, height, data).unwrap()
}

/// Maps a number between 0 and 1 to a color gradient.
///
/// `iters_to_escape` is the number to map. Returns an RGB color as a tuple of three bytes.
fn color_gradient(iters_to_escape: f64) -> (u8, u8, u8) {
    let g = sinebow();
    let t = (4.0 * iters_to_escape) % 1.0;
    let rgba = g.at(t).to_rgba8();
    (rgba[0], rgba[1], rgba[2])
}


fn main() {
    let (width, height) = (1200, 1200);
    let max_iter = 40000;

    let zoom_factor = 1.02;

    // let zoom_point = (-0.75, 0.109); // The point to zoom in on
    // let zoom_point = (-0.10109636384562, 0.95628651080914);
    // let zoom_point = (-0.77568377, 0.13646737);
    let zoom_point = (
        -1.74999841099374081749002483162428393452822172335808534616943930976364725846655540417646727085571962736578151132907961927190726789896685696750162524460775546580822744596887978637416593715319388030232414667046419863755743802804780843375, 
        -0.00000000000000165712469295418692325810961981279189026504290127375760405334498110850956047368308707050735960323397389547038231194872482690340369921750514146922400928554011996123112902000856666847088788158433995358406779259404221904755
    );

    let mut x_range = (-2.0 + zoom_point.0, 2.0 + zoom_point.0);
    let mut y_range = (-2.0 + zoom_point.1, 2.0 + zoom_point.1);


    for frame in 0..2000 {
        let start_time = Instant::now(); // Record the start time
        let mut img = render_mandelbrot(width, height, max_iter, x_range, y_range);

        invert(&mut img);

        let output_filename = format!("data/mandelbrot_set_{:04}.png", frame);

        let output_path = Path::new(&output_filename);
        if let Err(e) = img.save(&output_path) {
            eprintln!("Failed to save image: {}", e);
            std::process::exit(1);
        }

        // Update the x and y ranges to zoom in
        let x_center = (x_range.0 + x_range.1) / 2.0;
        let y_center = (y_range.0 + y_range.1) / 2.0;

        let x_range_width = (x_range.1 - x_range.0) / zoom_factor;
        let y_range_width = (y_range.1 - y_range.0) / zoom_factor;

        x_range = (
            x_center - x_range_width / 2.0,
            x_center + x_range_width / 2.0,
        );
        y_range = (
            y_center - y_range_width / 2.0,
            y_center + y_range_width / 2.0,
        );
        let elapsed_time = start_time.elapsed(); // Calculate the elapsed time
        println!(
            "Frame {} saved in {:.2?} seconds. X range: {:?}, Y range: {:?}: ",
            frame,
            elapsed_time.as_secs_f64(),
            x_range,
            y_range,
        );
    }
    let output = Command::new("ffmpeg")
        .arg("-framerate")
        .arg("30")
        .arg("-i")
        .arg("data/mandelbrot_set_%04d.png")
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("out.mp4")
        .output()
        .expect("Failed to execute command");

    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
}
