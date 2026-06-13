use super::*;

#[test]
fn test_percentage() {
    assert_eq!(percentage(0, 0), 0.0);
    assert_eq!(percentage(5, 0), 0.0);
    assert_eq!(percentage(5, 10), 50.0);
    assert_eq!(percentage(10, 10), 100.0);
}

#[test]
fn test_lerp() {
    assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    // test clamping
    assert_eq!(lerp(0.0, 10.0, -0.5), 0.0);
    assert_eq!(lerp(0.0, 10.0, 1.5), 10.0);
}

#[test]
fn test_hsl_to_rgb() {
    // Red: HSL(0, 1, 0.5) -> RGB(255, 0, 0)
    let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 0);

    // Green: HSL(120, 1, 0.5) -> RGB(0, 255, 0)
    let (r, g, b) = hsl_to_rgb(120.0, 1.0, 0.5);
    assert_eq!(r, 0);
    assert_eq!(g, 255);
    assert_eq!(b, 0);

    // Blue: HSL(240, 1, 0.5) -> RGB(0, 0, 255)
    let (r, g, b) = hsl_to_rgb(240.0, 1.0, 0.5);
    assert_eq!(r, 0);
    assert_eq!(g, 0);
    assert_eq!(b, 255);
}

#[test]
fn test_smooth_noise() {
    let n1 = smooth_noise(1.0, 0, 1.0, 1.0);
    let n2 = smooth_noise(1.0, 0, 1.0, 1.0);
    // Should be deterministic
    assert_eq!(n1, n2);

    let n3 = smooth_noise(2.0, 0, 1.0, 1.0);
    assert_ne!(n1, n3);
}

#[test]
fn test_lerp_negative() {
    // Lerping between negative numbers
    assert_eq!(lerp(-10.0, -20.0, 0.5), -15.0);
    assert_eq!(lerp(-10.0, 10.0, 0.5), 0.0);
}

#[test]
fn test_hsl_to_rgb_grayscale() {
    // Grayscale: HSL(0, 0, 0.5) -> RGB(127, 127, 127)
    let (r, g, b) = hsl_to_rgb(0.0, 0.0, 0.5);
    assert_eq!(r, 127);
    assert_eq!(g, 127);
    assert_eq!(b, 127);
}

#[test]
fn test_smooth_noise_zero_amplitude() {
    // 0 amplitude should yield 0.0
    let n = smooth_noise(12.34, 5, 0.0, 2.5);
    assert_eq!(n, 0.0);
}

