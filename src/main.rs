use image::{ImageBuffer, Rgb, RgbImage};
use std::{env, path::Path};

//const COLOURS: [(u8, u8, u8); 4] = [
//    // Purple
//    (159, 2, 209),
//    // Yellow
//    (234, 253, 0),
//    // Cyan
//    (0, 200, 167),
//    // Orange
//    (255, 116, 0),
//];

const COLOURS: [(u8, u8, u8); 3] = [
    // Deep blue
    (0, 33, 166),
    // Pink
    (250, 83, 252),
    // White
    (255, 255, 255),
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let res: (u32, u32) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let max: u64 = args[3].parse().unwrap();

    let target = (-1.99998588117, 0.);
    let zoom = (2 as f64).powf(32.8);

    let mut final_image = RgbImage::new(res.0, res.1);

    generate_mandelbrot(&mut final_image, max, target, zoom);

    save_image(final_image);
}

fn generate_mandelbrot(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    max: u64,
    target: (f64, f64),
    zoom: f64,
) {
    let scale = 4. / image.width().min(image.height()) as f64 / zoom;

    for x in 0..image.width() {
        for y in 0..image.height() {
            let pos = (
                (x as i32 - image.width() as i32 / 2) as f64 * scale + target.0,
                (y as i32 - image.height() as i32 / 2) as f64 * scale - target.1,
            );
            let val = calc_val(pos, max);

            if val == max {
                image.put_pixel(x, y, Rgb([0, 0, 0]));
            } else {
                image.put_pixel(x, y, gen_col(val, max));
            }
        }
    }
}

fn gen_col(val: u64, max: u64) -> Rgb<u8> {
    const SCL: f64 = 0.1;

    if val < max {
        let t = (val as f64 * SCL).rem_euclid(COLOURS.len() as f64);

        let col_a = COLOURS[t as usize];
        let col_b = COLOURS[t.ceil() as usize % COLOURS.len()];

        let col = lerp_col(col_a, col_b, t.fract());

        Rgb([col.0, col.1, col.2])
    } else {
        Rgb([0, 0, 0])
    }
}

fn calc_val(c: (f64, f64), max: u64) -> u64 {
    let mut z = (0., 0.);

    for m in 0..max {
        z = (z.0 * z.0 - z.1 * z.1 + c.0, 2. * z.0 * z.1 + c.1);
        if z.0 * z.0 + z.1 * z.1 > 4. {
            return m;
        }
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

fn lerp_col(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    (
        (a.0 as f64 * (1. - t) + b.0 as f64 * t) as u8,
        (a.1 as f64 * (1. - t) + b.1 as f64 * t) as u8,
        (a.2 as f64 * (1. - t) + b.2 as f64 * t) as u8,
    )
}
