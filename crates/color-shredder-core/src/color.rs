#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Color represented with normalized floating-point channels in the range [0.0, 1.0].
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0 };

    #[inline]
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    #[inline]
    pub fn to_rgb8(self) -> [u8; 3] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }

    #[inline]
    pub fn to_rgba8(self) -> [u8; 4] {
        let [r, g, b] = self.to_rgb8();
        [r, g, b, 255]
    }

    #[inline]
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    /// Squared Euclidean distance between two colors in RGB space.
    #[inline]
    pub fn dist_sq(self, other: Self) -> f32 {
        let dr = self.r - other.r;
        let dg = self.g - other.g;
        let db = self.b - other.b;
        dr * dr + dg * dg + db * db
    }

    /// Euclidean distance between two colors in RGB space.
    #[inline]
    pub fn dist(self, other: Self) -> f32 {
        self.dist_sq(other).sqrt()
    }
}

/// Color space selection for color palette generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ColorSpace {
    #[default]
    Rgb,
    Hsv,
    Hls,
}

/// Convert HSV (Hue [0, 1], Saturation [0, 1], Value [0, 1]) to RGB.
/// Matches Python's `colorsys.hsv_to_rgb`.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    if s <= 0.0 {
        return Color::new(v, v, v);
    }

    let h = (h % 1.0 + 1.0) % 1.0;
    let i = (h * 6.0).floor() as i32;
    let f = (h * 6.0) - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i % 6 {
        0 => Color::new(v, t, p),
        1 => Color::new(q, v, p),
        2 => Color::new(p, v, t),
        3 => Color::new(p, q, v),
        4 => Color::new(t, p, v),
        _ => Color::new(v, p, q),
    }
}

/// Convert HLS (Hue [0, 1], Lightness [0, 1], Saturation [0, 1]) to RGB.
/// Matches Python's `colorsys.hls_to_rgb`.
pub fn hls_to_rgb(h: f32, l: f32, s: f32) -> Color {
    let l = l.clamp(0.0, 1.0);
    let s = s.clamp(0.0, 1.0);
    if s <= 0.0 {
        return Color::new(l, l, l);
    }

    let m2 = if l <= 0.5 {
        l * (1.0 + s)
    } else {
        l + s - (l * s)
    };
    let m1 = 2.0 * l - m2;

    fn hue_to_rgb(m1: f32, m2: f32, mut h: f32) -> f32 {
        h = (h % 1.0 + 1.0) % 1.0;
        if h < 1.0 / 6.0 {
            m1 + (m2 - m1) * h * 6.0
        } else if h < 0.5 {
            m2
        } else if h < 2.0 / 3.0 {
            m1 + (m2 - m1) * (2.0 / 3.0 - h) * 6.0
        } else {
            m1
        }
    }

    Color::new(
        hue_to_rgb(m1, m2, h + 1.0 / 3.0),
        hue_to_rgb(m1, m2, h),
        hue_to_rgb(m1, m2, h - 1.0 / 3.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let black = Color::BLACK;
        let red = Color::RED;
        assert!((black.dist(red) - 1.0).abs() < 1e-6);

        let green = Color::GREEN;
        assert!((red.dist_sq(green) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_hsv_to_rgb() {
        let red_hsv = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((red_hsv.r - 1.0).abs() < 1e-5);
        assert!(red_hsv.g.abs() < 1e-5);
        assert!(red_hsv.b.abs() < 1e-5);

        let white_hsv = hsv_to_rgb(0.0, 0.0, 1.0);
        assert!((white_hsv.r - 1.0).abs() < 1e-5);
        assert!((white_hsv.g - 1.0).abs() < 1e-5);
        assert!((white_hsv.b - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_hls_to_rgb() {
        let red_hls = hls_to_rgb(0.0, 0.5, 1.0);
        assert!((red_hls.r - 1.0).abs() < 1e-5);
        assert!(red_hls.g.abs() < 1e-5);
        assert!(red_hls.b.abs() < 1e-5);
    }
}
