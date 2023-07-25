use image::{ImageBuffer, ImageResult, Rgb, RgbImage};
use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    let res: (u32, u32) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let max: u32 = args[3].parse().unwrap();

    let mut final_image = RgbImage::new(res.0, res.1);

    generate_mandelbrot(&mut final_image, max);

    save_image(final_image);
}

fn generate_mandelbrot(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, max: u32) {
    let scale = 4. / image.width().min(image.height()) as f64;

    for x in 0..image.width() {
        for y in 0..image.height() {
            let pos = (x as f64 * scale - 2., y as f64 * scale - 2.);
            let val = calc_val(pos, max);

            if val == max {
                image.put_pixel(x, y, Rgb([0, 0, 0]));
            } else {
                image.put_pixel(x, y, gen_col(val, max));
            }
        }
    }
}

fn gen_col(val: u32, max: u32) -> Rgb<u8> {
    const R_DESC: (f64, f64, f64) = (0.5, 64., 255.);
    const G_DESC: (f64, f64, f64) = (0.5, 0., 0.);
    const B_DESC: (f64, f64, f64) = (0.5, 64., 255.);

    let r = ((val as f64 * R_DESC.0).cos() + 1.) * (R_DESC.2 - R_DESC.1) + R_DESC.1;

    let g = ((val as f64 * G_DESC.0).cos() + 1.) * (G_DESC.2 - G_DESC.1) + G_DESC.1;
    let g = 32.;

    let b = ((val as f64 * B_DESC.0).sin() + 1.) * (B_DESC.2 - B_DESC.1) + B_DESC.1;

    Rgb([r as u8, g as u8, b as u8])
}

fn calc_val(c: (f64, f64), max: u32) -> u32 {
    let mut z = (0., 0.);

    for m in 0..max {
        if z.0 * z.0 + z.1 * z.1 > 4. {
            return m;
        }

        z = (z.0 * z.0 - z.1 * z.1 + c.0, 2. * z.0 * z.1 + c.1);
    }

    max
}

fn save_image(image: ImageBuffer<Rgb<u8>, Vec<u8>>) {
    let mut n = 0;

    'free_file_search: loop {
        match Path::new(&format!("images/{n}.png")).exists() {
            true => n += 1,
            false => break 'free_file_search,
        }
    }

    image
        .save(format!("images/{n}.png"))
        .expect("Failed to save image");
}
