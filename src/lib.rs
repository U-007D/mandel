#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![allow(non_camel_case_types)]
#![warn(missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unused_import_braces, unused_qualifications)]
#![deny(unused_must_use, overflowing_literals)]

extern crate num;
extern crate image;
extern crate rayon;
extern crate time;

use std::io::Write;
use std::str::FromStr;

use num::Complex;
use rayon::prelude::*;
use time::PreciseTime;

type GeneralError = Box<std::error::Error>;
type GeneralResult<T> = Result<T, GeneralError>;

#[cfg(test)]
mod unit_tests;

pub fn run(_args: Vec<String>) -> GeneralResult<()> {
    if _args.len() != 5 {
        writeln!(std::io::stderr(),
                 "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT")
            .unwrap();
        writeln!(std::io::stderr(),
                 "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
                 _args[0])
            .unwrap();
        std::process::exit(1);
    }
    let bounds = parse_pair(&_args[2], 'x')
        .expect("error parsing image dimensions");
    let upper_left = parse_pair(&_args[3], ',')
        .expect("error parsing upper left corner point");
    let lower_right = parse_pair(&_args[4], ',')
        .expect("error parsing lower right corner point");
    let mut pixels = vec![0; bounds.0 * bounds.1];

    let beg = PreciseTime::now();
    // Scope of slicing up `pixels` into horizontal bands.
    {
        let bands: Vec<(usize, &mut [u8])> = pixels
            .chunks_mut(bounds.0)
            .enumerate()
            .collect();
        bands.into_par_iter()
             .weight_max()
             .for_each(|(i, band)| {
                 let top = i;
                 let band_bounds = (bounds.0, 1);
                 let band_upper_left = pixel_to_point(bounds, (0, top),
                                                      upper_left, lower_right);
                 let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1),
                                                       upper_left, lower_right);
                 render(band, band_bounds, band_upper_left, band_lower_right);
             });
    }
    let end = PreciseTime::now();
    println!("{} ms", beg.to(end).num_milliseconds());
    write_bitmap(&_args[1], &pixels, bounds).expect("error writing PNG file");
    Ok(())
}

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed by `T::from_str`.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse
/// correctly, return `None`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None }
        } }
}

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius two centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escapes(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z*z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    return None;
}

/// Return the point on the complex plane corresponding to a given pixel in the
/// bitmap.
///
/// `bounds` is a pair giving the width and height of the bitmap. `pixel` is a
/// pair indicating a particular pixel in that bitmap. The `upper_left` and
/// `lower_right` parameters are points on the complex plane designating the
/// area our bitmap covers.
fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: (f64, f64),
                  lower_right: (f64, f64))
                  -> (f64, f64)
{
    // It might be nicer to find the position of the *middle* of the pixel,
    // instead of its upper left corner, but this is easier to write tests for.
    let (width, height) = (lower_right.0 - upper_left.0,
                           upper_left.1 - lower_right.1);
    (upper_left.0 + pixel.0 as f64 * width  / bounds.0 as f64,
     upper_left.1 - pixel.1 as f64 * height / bounds.1 as f64)
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
    ///
    /// The `bounds` argument gives the width and height of the buffer `pixels`,
    /// which holds one grayscale pixel per byte. The `upper_left` and `lower_right`
    /// arguments specify points on the complex plane corresponding to the upper
    /// left and lower right corners of the pixel buffer.
fn render(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: (f64, f64),
          lower_right: (f64, f64))
{
    assert!(pixels.len() == bounds.0 * bounds.1);
    for r in 0 .. bounds.1 {
        for c in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (c, r),
                                       upper_left, lower_right);
            pixels[r * bounds.0 + c] =
                match escapes(Complex { re: point.0, im: point.1 }, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;
/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_bitmap(filename: &str, pixels: &[u8], bounds: (usize, usize))
                -> Result<(), std::io::Error>
{
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels,
                   bounds.0 as u32, bounds.1 as u32,
                   ColorType::Gray(8))?;
    Ok(())
}
