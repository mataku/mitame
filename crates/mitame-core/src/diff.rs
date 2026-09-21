use image::{Rgba, RgbaImage};

const YIQ_MAX_DELTA: f64 = 35215.0;

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub diff_pixels: u64,
    pub total_pixels: u64,
    pub image: RgbaImage,
}

impl DiffResult {
    pub fn ratio(&self) -> f64 {
        if self.total_pixels == 0 {
            0.0
        } else {
            self.diff_pixels as f64 / self.total_pixels as f64
        }
    }
}

pub fn diff_images(baseline: &RgbaImage, current: &RgbaImage, pixel_tolerance: f64) -> DiffResult {
    debug_assert_eq!(baseline.dimensions(), current.dimensions());
    let (width, height) = baseline.dimensions();
    let max_delta = YIQ_MAX_DELTA * pixel_tolerance * pixel_tolerance;
    let mut out = RgbaImage::new(width, height);
    let mut diff_pixels = 0u64;
    for (x, y, b) in baseline.enumerate_pixels() {
        let c = current.get_pixel(x, y);
        if b == c {
            out.put_pixel(x, y, faded(b));
            continue;
        }
        if color_delta(b.0, c.0) > max_delta {
            diff_pixels += 1;
            out.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        } else {
            out.put_pixel(x, y, faded(b));
        }
    }
    DiffResult {
        diff_pixels,
        total_pixels: u64::from(width) * u64::from(height),
        image: out,
    }
}

pub fn color_delta(a: [u8; 4], b: [u8; 4]) -> f64 {
    if a == b {
        return 0.0;
    }
    let (r1, g1, b1) = blend_white(a);
    let (r2, g2, b2) = blend_white(b);
    let dy = rgb2y(r1, g1, b1) - rgb2y(r2, g2, b2);
    let di = rgb2i(r1, g1, b1) - rgb2i(r2, g2, b2);
    let dq = rgb2q(r1, g1, b1) - rgb2q(r2, g2, b2);
    0.5053 * dy * dy + 0.299 * di * di + 0.1957 * dq * dq
}

fn blend_white(p: [u8; 4]) -> (f64, f64, f64) {
    let a = f64::from(p[3]) / 255.0;
    let blend = |c: u8| 255.0 + (f64::from(c) - 255.0) * a;
    (blend(p[0]), blend(p[1]), blend(p[2]))
}

fn rgb2y(r: f64, g: f64, b: f64) -> f64 {
    r * 0.298_895_31 + g * 0.586_622_47 + b * 0.114_482_23
}

fn rgb2i(r: f64, g: f64, b: f64) -> f64 {
    r * 0.595_977_99 - g * 0.274_176_10 - b * 0.321_801_89
}

fn rgb2q(r: f64, g: f64, b: f64) -> f64 {
    r * 0.211_470_17 - g * 0.522_617_11 + b * 0.311_146_94
}

fn faded(p: &Rgba<u8>) -> Rgba<u8> {
    let (r, g, b) = blend_white(p.0);
    let y = rgb2y(r, g, b);
    let v = (255.0 + (y - 255.0) * 0.1).round().clamp(0.0, 255.0) as u8;
    Rgba([v, v, v, 255])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_pixels_have_zero_delta() {
        assert_eq!(color_delta([1, 2, 3, 255], [1, 2, 3, 255]), 0.0);
    }

    #[test]
    fn black_and_white_exceed_default_tolerance() {
        let max = YIQ_MAX_DELTA * 0.1 * 0.1;
        assert!(color_delta([0, 0, 0, 255], [255, 255, 255, 255]) > max);
    }

    #[test]
    fn near_identical_pixels_are_within_default_tolerance() {
        let max = YIQ_MAX_DELTA * 0.1 * 0.1;
        assert!(color_delta([100, 100, 100, 255], [102, 101, 100, 255]) < max);
    }

    #[test]
    fn diff_counts_only_pixels_over_tolerance() {
        let mut a = RgbaImage::from_pixel(4, 1, Rgba([0, 0, 0, 255]));
        let mut b = a.clone();
        b.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        b.put_pixel(1, 0, Rgba([2, 1, 0, 255]));
        a.put_pixel(2, 0, Rgba([10, 10, 10, 255]));
        b.put_pixel(2, 0, Rgba([10, 10, 10, 255]));
        let result = diff_images(&a, &b, 0.1);
        assert_eq!(result.diff_pixels, 1);
        assert_eq!(result.total_pixels, 4);
        assert_eq!(result.image.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
    }
}
