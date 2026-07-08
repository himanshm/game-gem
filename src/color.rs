//! Color types with comprehensive construction and manipulation.
//!
//! Unlike macroquad's bare `Color` struct, `game-gem` provides:
//! - Named color constants (CSS4 extended)
//! - Alpha-premultiplied blending helpers
//! - HSL/HSV conversion
//! - Lerp between colors
//! - Parse from hex strings

use std::str::FromStr;
use crate::math::FloatExt;

/// A color represented as linear RGBA floats (0.0–1.0).
///
/// All game-gem drawing functions accept this type.
/// Internal rendering may convert to premultiplied alpha.
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Color {
    /// Red channel (0.0–1.0).
    pub r: f32,
    /// Green channel (0.0–1.0).
    pub g: f32,
    /// Blue channel (0.0–1.0).
    pub b: f32,
    /// Alpha channel (0.0–1.0). 0.0 = fully transparent, 1.0 = fully opaque.
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA components (0.0–1.0).
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a fully opaque RGB color.
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create from 8-bit RGBA values (0–255).
    #[inline]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Create from 8-bit RGB values (0–255), fully opaque.
    #[inline]
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba8(r, g, b, 255)
    }

    /// Create from a hex string like `"#FF0080"` or `"FF008080"`.
    ///
    /// - 6 digits → RGB (fully opaque)
    /// - 8 digits → RGBA
    /// - Optional leading `#`
    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        let hex = hex.trim_start_matches('#');
        let val = u32::from_str_radix(hex, 16)
            .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?;

        match hex.len() {
            6 => Ok(Self::from_rgb8(
                ((val >> 16) & 0xFF) as u8,
                ((val >> 8) & 0xFF) as u8,
                (val & 0xFF) as u8,
            )),
            8 => Ok(Self::from_rgba8(
                ((val >> 24) & 0xFF) as u8,
                ((val >> 16) & 0xFF) as u8,
                ((val >> 8) & 0xFF) as u8,
                (val & 0xFF) as u8,
            )),
            _ => Err(ColorParseError::InvalidLength(hex.len())),
        }
    }

    /// Convert to a 32-bit RGBA integer (0xRRGGBBAA).
    pub fn to_rgba32(self) -> u32 {
        ((self.r.clamp(0.0, 1.0) * 255.0) as u32) << 24
            | ((self.g.clamp(0.0, 1.0) * 255.0) as u32) << 16
            | ((self.b.clamp(0.0, 1.0) * 255.0) as u32) << 8
            | (self.a.clamp(0.0, 1.0) * 255.0) as u32
    }

    /// Convert to premultiplied alpha form.
    #[inline]
    pub fn premultiplied(self) -> Self {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    /// Get the luminance (perceived brightness).
    #[inline]
    pub fn luminance(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Lighten the color by `amount` (0.0–1.0).
    pub fn lightened(self, amount: f32) -> Self {
        Self {
            r: (self.r + (1.0 - self.r) * amount).min(1.0),
            g: (self.g + (1.0 - self.g) * amount).min(1.0),
            b: (self.b + (1.0 - self.b) * amount).min(1.0),
            a: self.a,
        }
    }

    /// Darken the color by `amount` (0.0–1.0).
    pub fn darkened(self, amount: f32) -> Self {
        Self {
            r: (self.r * (1.0 - amount)).max(0.0),
            g: (self.g * (1.0 - amount)).max(0.0),
            b: (self.b * (1.0 - amount)).max(0.0),
            a: self.a,
        }
    }

    /// Return the color with a new alpha.
    #[inline]
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Linear interpolation between two colors.
    #[inline]
    pub fn lerp(self, other: Color, t: f32) -> Color {
        Color {
            r: self.r.lerp(other.r, t),
            g: self.g.lerp(other.g, t),
            b: self.b.lerp(other.b, t),
            a: self.a.lerp(other.a, t),
        }
    }

    /// Convert to HSLA (hue: 0–360, sat/light/alpha: 0–1).
    pub fn to_hsla(self) -> (f32, f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let lightness = (max + min) / 2.0;

        if (max - min).abs() < 1e-6 {
            return (0.0, 0.0, lightness, self.a);
        }

        let d = max - min;
        let saturation = if lightness > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let hue = if (max - self.r).abs() < 1e-6 {
            ((self.g - self.b) / d + (if self.g < self.b { 6.0 } else { 0.0 })) * 60.0
        } else if (max - self.g).abs() < 1e-6 {
            ((self.b - self.r) / d + 2.0) * 60.0
        } else {
            ((self.r - self.g) / d + 4.0) * 60.0
        };

        (hue, saturation, lightness, self.a)
    }

    /// Create from HSLA values (hue: 0–360, sat/light/alpha: 0–1).
    pub fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        if (s - 0.0).abs() < 1e-6 {
            return Color::new(l, l, l, a);
        }

        let hue2 = h / 60.0;
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - (hue2 % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = match hue2 as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color::new(r1 + m, g1 + m, b1 + m, a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color(#{:02X}{:02X}{:02X}{:02X})",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }
}

impl FromStr for Color {
    type Err = ColorParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Color::from_hex(s)
    }
}

/// Error type for color parsing.
#[derive(Debug, Clone)]
pub enum ColorParseError {
    InvalidHex(String),
    InvalidLength(usize),
}

impl std::fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorParseError::InvalidHex(h) => write!(f, "Invalid hex color: {}", h),
            ColorParseError::InvalidLength(len) => {
                write!(f, "Expected 6 or 8 hex digits, got {}", len)
            }
        }
    }
}

impl std::error::Error for ColorParseError {}

// --- Named color constants (CSS4 extended + common game colors) ---

impl Color {
    pub const WHITE:   Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK:   Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const RED:     Color = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN:   Color = Color::new(0.0, 0.8, 0.0, 1.0);
    pub const BLUE:    Color = Color::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW:  Color = Color::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN:    Color = Color::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::new(1.0, 0.0, 1.0, 1.0);
    pub const ORANGE:  Color = Color::new(1.0, 0.5, 0.0, 1.0);

    // Shades
    pub const GRAY:    Color = Color::new(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GRAY: Color = Color::new(0.75, 0.75, 0.75, 1.0);
    pub const DARK_GRAY:  Color = Color::new(0.25, 0.25, 0.25, 1.0);

    // Transparent
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

    // Game-specific common colors
    pub const SKY_BLUE:  Color = Color::new(0.53, 0.81, 0.92, 1.0);
    pub const GOLD:      Color = Color::new(1.0, 0.84, 0.0, 1.0);
    pub const CORAL:     Color = Color::new(1.0, 0.5, 0.31, 1.0);
    pub const SALMON:    Color = Color::new(0.98, 0.5, 0.45, 1.0);
    pub const LIME:      Color = Color::new(0.0, 1.0, 0.0, 1.0);
    pub const PURPLE:    Color = Color::new(0.5, 0.0, 0.5, 1.0);
    pub const PINK:      Color = Color::new(1.0, 0.75, 0.8, 1.0);
    pub const TEAL:      Color = Color::new(0.0, 0.5, 0.5, 1.0);
    pub const NAVY:      Color = Color::new(0.0, 0.0, 0.5, 1.0);
    pub const MAROON:    Color = Color::new(0.5, 0.0, 0.0, 1.0);
    pub const OLIVE:     Color = Color::new(0.5, 0.5, 0.0, 1.0);
    pub const AQUA:      Color = Color::new(0.0, 1.0, 1.0, 1.0);
    pub const INDIGO:    Color = Color::new(0.29, 0.0, 0.51, 1.0);
    pub const VIOLET:    Color = Color::new(0.58, 0.0, 0.83, 1.0);
    pub const CRIMSON:   Color = Color::new(0.86, 0.08, 0.24, 1.0);
    pub const TURQUOISE: Color = Color::new(0.25, 0.88, 0.82, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        let c = Color::from_hex("#FF0080").unwrap();
        assert!((c.r - 1.0).abs() < 1e-6);
        assert!((c.g - 0.0).abs() < 1e-6);
        assert!((c.b - 0.502).abs() < 0.01);
        assert!((c.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-6);
        assert!((mid.g - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hsla_roundtrip() {
        let original = Color::new(0.8, 0.3, 0.5, 0.9);
        let (h, s, l, a) = original.to_hsla();
        let roundtrip = Color::from_hsla(h, s, l, a);
        assert!((roundtrip.r - original.r).abs() < 0.01);
        assert!((roundtrip.g - original.g).abs() < 0.01);
        assert!((roundtrip.b - original.b).abs() < 0.01);
    }
}