//! WCAG 2.2 relative luminance and contrast ratio.

use super::tokens::Rgb;

fn channel(c: u8) -> f64 {
    let v = f64::from(c) / 255.0;
    if v <= 0.039_28 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(c: Rgb) -> f64 {
    0.2126 * channel(c.0) + 0.7152 * channel(c.1) + 0.0722 * channel(c.2)
}

/// Contrast ratio between two colors, 1.0 to 21.0, symmetric.
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white_is_21() {
        assert!((contrast(Rgb(0, 0, 0), Rgb(255, 255, 255)) - 21.0).abs() < 0.01);
    }

    #[test]
    fn grey_777_on_white() {
        let r = contrast(Rgb(0x77, 0x77, 0x77), Rgb(255, 255, 255));
        assert!((r - 4.48).abs() < 0.01, "{r}");
    }

    #[test]
    fn symmetric() {
        let (a, b) = (Rgb(10, 200, 30), Rgb(250, 3, 90));
        assert!((contrast(a, b) - contrast(b, a)).abs() < f64::EPSILON);
    }
}
