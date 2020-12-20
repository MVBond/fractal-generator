use std::cmp;
use std::sync::mpsc;
use std::thread;
use clap::Clap;
use num::Complex;
use options::Options;
use rand::Rng;

mod colour;
mod options;

const MAX_VALUE_EXTENT: f64 = 2.0;
const MAX_ITERATIONS: u32 = 1000;

fn main() {
    let opts: Options = Options::parse();

    let centre = Complex::new(opts.centre_real, opts.centre_imaginary);
    let scale = 2. / opts.zoom / (cmp::min(opts.width, opts.height) as f64);

    let (tx, rx) = mpsc::channel();

    let mut buffer: Vec<u8> = vec!();
    buffer.resize((opts.width * opts.height * 3) as usize, 0);

    let num_threads = cmp::min(opts.threads - 1, opts.height);

    for row_number in 0..num_threads {
        spawn_thread(row_number, centre, scale, opts.width, opts.height, opts.samples_per_pixel, &tx);
    }

    let mut finished = 0;

    if num_threads < opts.height {
        for row_number in num_threads..opts.height {
            let (complete_row, complete_row_number) = rx.recv().unwrap();
            spawn_thread(row_number, centre, scale, opts.width, opts.height, opts.samples_per_pixel, &tx);
            add_row_to_buffer(&mut buffer, complete_row, complete_row_number);
            finished += 1;
            print!("Rendered: {}%\r", finished * 100 / opts.height);
        }
    }

    // Drop the MPSC tx so that the receiver will stop listening.
    drop(tx);

    for (row, row_number) in rx {
        add_row_to_buffer(&mut buffer, row, row_number);
        finished += 1;
        print!("Rendered: {}%\r", finished * 100 / opts.height);
    }

    image::save_buffer(opts.output, &buffer, opts.width, opts.height, image::ColorType::Rgb8).unwrap();
}

fn spawn_thread(row_number: u32,
                centre: Complex<f64>,
                scale: f64,
                width: u32,
                height: u32,
                num_samples: u32,
                tx: &mpsc::Sender<(Vec<u8>, u32)>) {
    let new_tx = mpsc::Sender::clone(&tx);
    thread::spawn(move || {
        let row = render_row(row_number, centre, scale, width, height, num_samples);
        new_tx.send((row, row_number)).unwrap();
    });
}

fn render_row(line_number: u32,
              centre: Complex<f64>,
              scale: f64,
              width: u32,
              height: u32,
              num_samples: u32)
              -> Vec<u8> {
    let mut row: Vec<u8> = Vec::with_capacity((width * 3) as usize);

    let mut rng = rand::thread_rng();

    for pixel_number in 0..width {
        let mut rgb: (u32, u32, u32) = (0, 0, 0);

        for _ in 0..num_samples {
            let real = ((2.0 * ((pixel_number as f64) + rng.gen_range(0.0, 1.0)) - (width as f64))) * scale + centre.re;
            let imaginary =
                ((2.0 * ((line_number as f64) + rng.gen_range(0.0, 1.0)) - (height as f64))) * scale + centre.im;
            let complex_number = Complex::new(real, imaginary);
            let value = mandelbrot_iteration(complex_number);

            let colour = match value {
                Some(value) => colour::hsl_to_rgb(value, 1.0, 0.5),
                // White.
                None => (255, 255, 255),
            };

            rgb.0 += colour.0 as u32;
            rgb.1 += colour.1 as u32;
            rgb.2 += colour.2 as u32;
        }

        row.push((rgb.0 / num_samples) as u8);
        row.push((rgb.1 / num_samples) as u8);
        row.push((rgb.2 / num_samples) as u8);
    }

    row
}

fn add_row_to_buffer(buffer: &mut Vec<u8>, row: Vec<u8>, row_number: u32) {
    let row_length = row.len();
    for row_index in 0..row_length {
        buffer[(row_number as usize) * row_length + row_index] = row[row_index];
    }
}

fn mandelbrot_iteration(c: Complex<f64>) -> Option<f64> {
    const MAX_NORM: f64 = MAX_VALUE_EXTENT * MAX_VALUE_EXTENT;

    let mut z = Complex::new(0., 0.);

    let mut norm;

    for iteration in 0..MAX_ITERATIONS {
        z = z * z + c;

        norm = z.norm();

        if norm >= MAX_NORM {
            let value = (iteration as f64) * norm / (MAX_ITERATIONS as f64);
            return Some(value)
        }
    }

    None
}
