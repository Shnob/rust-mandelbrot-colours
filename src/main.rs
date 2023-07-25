use image::{ImageBuffer, Rgb, RgbImage};
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
            let pos = (
                (x as i32 - image.width() as i32 / 2) as f64 * scale,
                (y as i32 - image.height() as i32 / 2) as f64 * scale,
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

fn gen_col(val: u32, max: u32) -> Rgb<u8> {
    let val_i32 = val as i32;
    const VIEW_MOD: i32 = 20;
    const COL_OFFSET: f64 = -3.29;
    const COL_SPACE_FAC: f64 = 2.;

    let col =
        (val_i32 % VIEW_MOD) as f64 / VIEW_MOD as f64 * std::f64::consts::PI * 2. + COL_OFFSET;

    if val < max {
        Rgb([
            ((col.sin() * 0.5 + 0.5) * 255.) as u8,
            (((col * COL_SPACE_FAC).sin() / 2. + 0.5) * 255.) as u8,
            ((col.cos() * 0.5 + 0.5) * 255.) as u8,
        ])
    } else {
        Rgb([0, 0, 0])
    }
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
