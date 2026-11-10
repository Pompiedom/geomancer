//! Rose: the rose curve r = R·cos(n/d·θ).

use crate::Shape;
use geomancer_common::{Value, curve, flag, num, placement, shape};
use std::f64::consts::PI;

pub const SHAPE: Shape = Shape {
    id: "rose",
    label: "Rose",
    params: r#"{
      "radius": {"type": "number", "min": 1, "max": 5000, "default": 100, "unit": "pt"},
      "n": {"type": "int", "min": 1, "max": 50, "default": 4,
            "help": "Petals: with d = 1, an odd n gives n petals and an even n gives 2n."},
      "d": {"type": "int", "min": 1, "max": 50, "default": 1,
            "help": "With d > 1 the petals overlap; the curve r = R·cos(n/d·θ) closes after d or 2d half-turns."},
      "rotation": {"type": "number", "min": -360, "max": 360, "default": -90, "unit": "°"},
      "fill": {"type": "bool", "default": false, "help": "A black fill instead of a 1 pt black stroke."}
    }"#,
    run,
};

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn run(input: &Value, params: &Value) -> Result<Vec<Value>, String> {
    let radius = num(params, "radius", 100.0);
    let (n, d) = (num(params, "n", 4.0).round().max(1.0) as u64, num(params, "d", 1.0).round().max(1.0) as u64);
    let g = gcd(n, d);
    let (n, d) = (n / g, d / g);
    let k = n as f64 / d as f64;
    // The curve closes after π·d when n and d are both odd, else after 2π·d.
    let span = if (n * d) % 2 == 1 { PI * d as f64 } else { 2.0 * PI * d as f64 };
    // A petal spans π/k; 12 segments a petal.
    let segments = ((span * k / PI * 12.0).ceil() as usize).clamp(16, 20_000);
    let rot = num(params, "rotation", -90.0).to_radians();
    let c = placement(input, params);
    let path = curve(
        |t| {
            let r = radius * (k * t).cos();
            (c.0 + r * (t + rot).cos(), c.1 + r * (t + rot).sin())
        },
        0.0,
        span,
        segments,
        true,
    );
    Ok(vec![shape("Rose", vec![path], flag(params, "fill"))])
}
