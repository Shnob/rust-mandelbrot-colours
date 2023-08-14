use image::{ImageBuffer, Rgb, RgbImage};
use little_exif::{exif_tag::ExifTag, metadata::Metadata};
use std::{
    env,
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

const COLOURS: [(u8, u8, u8); 3] = [
    // Deep blue
    (0, 33, 166),
    // Pink
    (250, 83, 252),
    // White
    (255, 255, 255),
];

const SMOOTH: bool = true;
// This value is the value at which a point is considered outside the set.
// Higher values take longer, but provide better smoothing.
const B: f64 = 1000.;
// Dimentions of the equation, should be 2 for mandelbrot set.
// This stuff is from https://iquilezles.org/articles/msetsmooth/
const D: f64 = 2.;

fn main() {
    let args: Vec<String> = env::args().collect();
    let res: (u32, u32) = (args[1].parse().unwrap(), args[2].parse().unwrap());
    let max: u64 = args[3].parse().unwrap_or_else(|_e| {
        println!("No max value provided; Assuming 100");
        100
    });
    let sampling = args[4].parse().unwrap_or_else(|_e| {
        println!("No multi-sampling value provided; Assuming 1");
        1
    });

    // Void Spiral
    let target = (-0.7765927806, -0.1366408558);
    let zoom = (2 as f64).powf(29.9);
    const JULIA: Option<(f64, f64)> = None;

    let rendered_image_am = Arc::new(Mutex::new(RgbImage::new(
        res.0 * sampling,
        res.1 * sampling,
    )));

    let start_time = Instant::now();

    generate_mandelbrot(Arc::clone(&rendered_image_am), max, target, zoom, JULIA);

    let time_taken = start_time.elapsed();
    let time_taken = time_taken.as_secs_f64();

    println!("Time taken: {time_taken}s");

    let rendered_image = rendered_image_am.lock().unwrap();

    let mut final_image = RgbImage::new(res.0, res.1);

    let n = (sampling * sampling) as u16;
    for x in 0..final_image.width() {
        for y in 0..final_image.height() {
            let mut col = (0, 0, 0);
            for x in (sampling * x)..(sampling * x + sampling) {
                for y in (sampling * y)..(sampling * y + sampling) {
                    let pix = rendered_image.get_pixel(x, y);
                    col = (
                        col.0 + pix[0] as u16,
                        col.1 + pix[1] as u16,
                        col.2 + pix[2] as u16,
                    );
                }
            }
            col = (col.0 / n, col.1 / n, col.2 / n);
            final_image.put_pixel(x, y, Rgb([col.0 as u8, col.1 as u8, col.2 as u8]));
        }
    }

    save_image(final_image, target, zoom, JULIA, &COLOURS, max, sampling);
}

enum CalcValue {
    Inside,
    Outside(f64),
}

fn generate_mandelbrot(
    image: Arc<Mutex<ImageBuffer<Rgb<u8>, Vec<u8>>>>,
    max: u64,
    target: (f64, f64),
    zoom: f64,
    julia: Option<(f64, f64)>,
) {
    let wid = image.lock().unwrap().width();
    let hit = image.lock().unwrap().height();
    let scale = 4. / wid.min(hit) as f64 / zoom;

    let mut threads = vec![];

    for x in 0..wid {
        let image = Arc::clone(&image);

        threads.push(thread::spawn(move || {
            for y in 0..hit {
                let pos = (
                    (x as i32 - wid as i32 / 2) as f64 * scale + target.0,
                    (y as i32 - hit as i32 / 2) as f64 * scale + target.1,
                );

                let val = match julia {
                    Some(c) => calc_val_julia(pos, c, max),
                    None => calc_val(pos, max),
                };

                match val {
                    CalcValue::Inside => {
                        image.lock().unwrap().put_pixel(x, y, Rgb([0, 0, 0]));
                    }
                    CalcValue::Outside(val) => {
                        image.lock().unwrap().put_pixel(x, y, gen_col(val));
                    }
                }
            }
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}

fn gen_col(val: f64) -> Rgb<u8> {
    const SCL: f64 = 0.1;

    let t = (val as f64 * SCL).rem_euclid(COLOURS.len() as f64);

    let col_a = COLOURS[t as usize];
    let col_b = COLOURS[t.ceil() as usize % COLOURS.len()];

    let col = lerp_col(col_a, col_b, t.fract());

    Rgb([col.0, col.1, col.2])
}

fn calc_val(c: (f64, f64), max: u64) -> CalcValue {
    let mut z = (0., 0.);

    for m in 0..max {
        z = (z.0 * z.0 - z.1 * z.1 + c.0, 2. * z.0 * z.1 + c.1);
        if z.0 * z.0 + z.1 * z.1 > B * B {
            match SMOOTH {
                true => {
                    let z_abs = f64::sqrt(z.0 * z.0 + z.1 * z.1);
                    let val = m as f64 - ((z_abs.ln() / B.ln()).ln()) / f64::ln(D);
                    return CalcValue::Outside(val);
                }
                false => {
                    return CalcValue::Outside(m as f64);
                }
            }
        }
    }

    CalcValue::Inside
}

fn calc_val_julia(mut z: (f64, f64), c: (f64, f64), max: u64) -> CalcValue {
    for m in 0..max {
        z = (z.0 * z.0 - z.1 * z.1 + c.0, 2. * z.0 * z.1 + c.1);
        if z.0 * z.0 + z.1 * z.1 > B * B {
            match SMOOTH {
                true => {
                    let z_abs = f64::sqrt(z.0 * z.0 + z.1 * z.1);
                    let val = m as f64 - ((z_abs.ln() / B.ln()).ln()) / f64::ln(D);
                    return CalcValue::Outside(val);
                }
                false => {
                    return CalcValue::Outside(m as f64);
                }
            }
        }
    }

    CalcValue::Inside
}

fn save_image(
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pos: (f64, f64),
    zoom: f64,
    julia: Option<(f64, f64)>,
    colours: &[(u8, u8, u8)],
    max: u64,
    sampling: u32,
) {
    let mut n = 0;

    'free_file_search: loop {
        match Path::new(&format!("images/{n}.png")).exists() {
            true => n += 1,
            false => break 'free_file_search,
        }
    }

    let file_path = format!("images/{n}.png");

    image.save(&file_path).expect("Failed to save image");

    let mut metadata = Metadata::new();

    metadata.set_tag(ExifTag::ImageDescription(generate_metadata(
        pos, zoom, julia, colours, max, sampling,
    )));

    metadata
        .write_to_file(Path::new(&file_path))
        .expect("Unable to write metadata to file");
}

fn generate_metadata(
    pos: (f64, f64),
    zoom: f64,
    julia: Option<(f64, f64)>,
    colours: &[(u8, u8, u8)],
    max: u64,
    sampling: u32,
) -> String {
    let x = pos.0;
    let y = pos.1;

    let julia_text = match julia {
        None => "n/a".into(),
        Some(julia) => {
            let x = julia.0;
            let y = julia.1;
            format!("{x} + {y}i")
        }
    };

    let colours_text = {
        let mut text = String::new();

        for colour in colours {
            let r = colour.0;
            let g = colour.1;
            let b = colour.2;

            text = format!("{text}\n\t{r}, {g}, {b}");
        }

        text
    };

    format!(
        "Target: {x} + {y}i
zoom: {zoom}
julia: {julia_text}
colours: {colours_text}
max: {max}
sampling: {sampling}"
    )
    .into()
}

fn lerp_col(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    (
        (a.0 as f64 * (1. - t) + b.0 as f64 * t) as u8,
        (a.1 as f64 * (1. - t) + b.1 as f64 * t) as u8,
        (a.2 as f64 * (1. - t) + b.2 as f64 * t) as u8,
    )
}
