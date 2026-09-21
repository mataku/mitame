use image::{Rgba, RgbaImage};

const YIQ_MAX_DELTA: f64 = 35215.0;
const DIFF_COLOR: Rgba<u8> = Rgba([255, 0, 0, 255]);
const AA_COLOR: Rgba<u8> = Rgba([255, 200, 0, 255]);

#[derive(Debug, Clone, Copy)]
pub struct DiffOptions {
    pub pixel_tolerance: f64,
    pub ignore_anti_aliasing: bool,
}

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub diff_pixels: u64,
    pub anti_aliased_pixels: u64,
    pub total_pixels: u64,
    pub bounds: Option<(u32, u32, u32, u32)>,
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

pub fn diff_images(baseline: &RgbaImage, current: &RgbaImage, options: DiffOptions) -> DiffResult {
    debug_assert_eq!(baseline.dimensions(), current.dimensions());
    let (width, height) = baseline.dimensions();
    let max_delta = YIQ_MAX_DELTA * options.pixel_tolerance * options.pixel_tolerance;
    let mut out = RgbaImage::new(width, height);
    let mut diff_pixels = 0u64;
    let mut anti_aliased_pixels = 0u64;
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    for (x, y, b) in baseline.enumerate_pixels() {
        let c = current.get_pixel(x, y);
        if b == c || color_delta(b.0, c.0) <= max_delta {
            out.put_pixel(x, y, faded(b));
            continue;
        }
        if options.ignore_anti_aliasing
            && (is_anti_aliased(baseline, current, x, y)
                || is_anti_aliased(current, baseline, x, y))
        {
            anti_aliased_pixels += 1;
            out.put_pixel(x, y, AA_COLOR);
            continue;
        }
        diff_pixels += 1;
        bounds = Some(match bounds {
            None => (x, y, x, y),
            Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
        });
        out.put_pixel(x, y, DIFF_COLOR);
    }
    if let Some(b) = bounds {
        draw_bounds(&mut out, b);
    }
    DiffResult {
        diff_pixels,
        anti_aliased_pixels,
        total_pixels: u64::from(width) * u64::from(height),
        bounds,
        image: out,
    }
}

const BOUNDS_COLOR: Rgba<u8> = Rgba([255, 0, 0, 160]);
const BOUNDS_PADDING: u32 = 6;

fn draw_bounds(img: &mut RgbaImage, (x0, y0, x1, y1): (u32, u32, u32, u32)) {
    let (w, h) = img.dimensions();
    let left = x0.saturating_sub(BOUNDS_PADDING);
    let top = y0.saturating_sub(BOUNDS_PADDING);
    let right = (x1 + BOUNDS_PADDING).min(w - 1);
    let bottom = (y1 + BOUNDS_PADDING).min(h - 1);
    for x in left..=right {
        for y in [top, bottom] {
            blend(img, x, y);
        }
    }
    for y in top..=bottom {
        for x in [left, right] {
            blend(img, x, y);
        }
    }
}

fn blend(img: &mut RgbaImage, x: u32, y: u32) {
    let p = img.get_pixel_mut(x, y);
    let a = f64::from(BOUNDS_COLOR[3]) / 255.0;
    for i in 0..3 {
        p[i] = (f64::from(p[i]) * (1.0 - a) + f64::from(BOUNDS_COLOR[i]) * a).round() as u8;
    }
    p[3] = 255;
}

fn is_anti_aliased(img: &RgbaImage, other: &RgbaImage, x1: u32, y1: u32) -> bool {
    let (width, height) = img.dimensions();
    let x0 = x1.saturating_sub(1);
    let y0 = y1.saturating_sub(1);
    let x2 = (x1 + 1).min(width - 1);
    let y2 = (y1 + 1).min(height - 1);
    let center = img.get_pixel(x1, y1).0;
    let mut zeroes = u32::from(x1 == x0 || x1 == x2 || y1 == y0 || y1 == y2);
    let mut min = 0.0f64;
    let mut max = 0.0f64;
    let mut min_pos = None;
    let mut max_pos = None;
    for x in x0..=x2 {
        for y in y0..=y2 {
            if x == x1 && y == y1 {
                continue;
            }
            let delta = brightness_delta(center, img.get_pixel(x, y).0);
            if delta == 0.0 {
                zeroes += 1;
                if zeroes > 2 {
                    return false;
                }
            } else if delta < min {
                min = delta;
                min_pos = Some((x, y));
            } else if delta > max {
                max = delta;
                max_pos = Some((x, y));
            }
        }
    }
    let (Some(min_pos), Some(max_pos)) = (min_pos, max_pos) else {
        return false;
    };
    (has_many_siblings(img, min_pos) && has_many_siblings(other, min_pos))
        || (has_many_siblings(img, max_pos) && has_many_siblings(other, max_pos))
}

fn has_many_siblings(img: &RgbaImage, (x1, y1): (u32, u32)) -> bool {
    let (width, height) = img.dimensions();
    let x0 = x1.saturating_sub(1);
    let y0 = y1.saturating_sub(1);
    let x2 = (x1 + 1).min(width - 1);
    let y2 = (y1 + 1).min(height - 1);
    let center = img.get_pixel(x1, y1);
    let mut zeroes = u32::from(x1 == x0 || x1 == x2 || y1 == y0 || y1 == y2);
    for x in x0..=x2 {
        for y in y0..=y2 {
            if x == x1 && y == y1 {
                continue;
            }
            if img.get_pixel(x, y) == center {
                zeroes += 1;
                if zeroes > 2 {
                    return true;
                }
            }
        }
    }
    false
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

fn brightness_delta(a: [u8; 4], b: [u8; 4]) -> f64 {
    if a == b {
        return 0.0;
    }
    let (r1, g1, b1) = blend_white(a);
    let (r2, g2, b2) = blend_white(b);
    rgb2y(r1, g1, b1) - rgb2y(r2, g2, b2)
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

    const STRICT: DiffOptions = DiffOptions {
        pixel_tolerance: 0.1,
        ignore_anti_aliasing: false,
    };
    const LENIENT: DiffOptions = DiffOptions {
        pixel_tolerance: 0.1,
        ignore_anti_aliasing: true,
    };

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
        let result = diff_images(&a, &b, STRICT);
        assert_eq!(result.diff_pixels, 1);
        assert_eq!(result.total_pixels, 4);
        assert_eq!(result.image.get_pixel(0, 0), &DIFF_COLOR);
        assert_eq!(result.bounds, Some((0, 0, 0, 0)));
    }

    fn edge_image(edge_gray: u8) -> RgbaImage {
        let mut img = RgbaImage::new(5, 5);
        for (x, _, p) in img.enumerate_pixels_mut() {
            *p = match x {
                0 | 1 => Rgba([0, 0, 0, 255]),
                2 => Rgba([edge_gray, edge_gray, edge_gray, 255]),
                _ => Rgba([255, 255, 255, 255]),
            };
        }
        img
    }

    #[test]
    fn anti_aliased_edge_is_ignored_when_enabled() {
        let a = edge_image(128);
        let b = edge_image(60);
        let strict = diff_images(&a, &b, STRICT);
        assert_eq!(strict.diff_pixels, 5);
        let lenient = diff_images(&a, &b, LENIENT);
        assert_eq!(lenient.diff_pixels, 0);
        assert_eq!(lenient.anti_aliased_pixels, 5);
        assert_eq!(lenient.image.get_pixel(2, 2), &AA_COLOR);
    }

    #[test]
    fn solid_block_change_is_not_anti_aliasing() {
        let a = RgbaImage::from_pixel(6, 6, Rgba([0, 0, 0, 255]));
        let mut b = a.clone();
        for x in 2..4 {
            for y in 2..4 {
                b.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let lenient = diff_images(&a, &b, LENIENT);
        assert_eq!(lenient.diff_pixels, 4);
        assert_eq!(lenient.anti_aliased_pixels, 0);
    }

    #[test]
    fn bounds_cover_all_differing_pixels_and_are_outlined() {
        let a = RgbaImage::from_pixel(40, 40, Rgba([0, 0, 0, 255]));
        let mut b = a.clone();
        b.put_pixel(10, 12, Rgba([255, 255, 255, 255]));
        b.put_pixel(20, 25, Rgba([255, 255, 255, 255]));
        let result = diff_images(&a, &b, STRICT);
        assert_eq!(result.bounds, Some((10, 12, 20, 25)));
        let corner = result
            .image
            .get_pixel(10 - BOUNDS_PADDING, 12 - BOUNDS_PADDING);
        assert!(corner[0] > 100 && corner[3] == 255);
        assert_eq!(
            result.image.get_pixel(30, 30),
            &faded(&Rgba([0, 0, 0, 255]))
        );
    }
}
