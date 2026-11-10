//! Wave: a horizontal sine, triangle, square or sawtooth wave of a given length.

use crate::Shape;
use geomancer_common::{P, Value, choice, curve, num, placement, polyline, shape};
use std::f64::consts::TAU;

pub const SHAPE: Shape = Shape {
    id: "wave",
    label: "Wave",
    params: r#"{
      "form": {"type": "choice", "options": ["sine", "triangle", "square", "sawtooth"], "default": "sine"},
      "length": {"type": "number", "min": 1, "max": 10000, "default": 300, "unit": "pt"},
      "amplitude": {"type": "number", "min": 0, "max": 5000, "default": 20, "unit": "pt",
                    "help": "From the centre line to a crest."},
      "cycles": {"type": "number", "min": 0.25, "max": 500, "default": 5},
      "phase": {"type": "number", "min": -360, "max": 360, "default": 0, "unit": "°",
                "help": "Where in its cycle the wave starts: 0° on the centre line going up, 90° at a crest."}
    }"#,
    run,
};

fn run(input: &Value, params: &Value) -> Result<Vec<Value>, String> {
    let length = num(params, "length", 300.0);
    let amp = num(params, "amplitude", 20.0);
    let cycles = num(params, "cycles", 5.0);
    let phase = num(params, "phase", 0.0) / 360.0;
    let c = placement(input, params);
    let x0 = c.0 - length / 2.0;
    // The point at cycle position u (in cycles from the start), y up from the centre line.
    let at = |u: f64, y: f64| -> P { (x0 + length * u / cycles, c.1 - amp * y) };
    let form = choice(params, "form", "sine");
    let path = match form {
        "sine" => {
            // 8 segments a cycle keep the cubic within 0.03 % of the amplitude.
            let segments = ((8.0 * cycles).ceil() as usize).max(2);
            curve(|u| at(u, (TAU * (u + phase)).sin()), 0.0, cycles, segments, false)
        }
        _ => {
            // Corners where the wave turns or jumps, in cycle positions relative to the phase.
            let (turns, jump): (&[f64], bool) = match form {
                "triangle" => (&[0.25, 0.75], false),
                "square" => (&[0.0, 0.5], true),
                _ => (&[0.5], true),
            };
            let y = |t: f64| -> f64 {
                let t = t.rem_euclid(1.0);
                match form {
                    "triangle" => 1.0 - 4.0 * (t - 0.25).abs().min((t - 1.25).abs()),
                    "square" => if t < 0.5 { 1.0 } else { -1.0 },
                    _ => if t < 0.5 { 2.0 * t } else { 2.0 * t - 2.0 },
                }
            };
            let mut points = vec![at(0.0, y(phase))];
            let first = (phase.floor() - 1.0) as i64;
            for k in first..=(phase + cycles).ceil() as i64 + 1 {
                for &t in turns {
                    // Corner at cycle position u = k + t − phase along the path.
                    let u = k as f64 + t - phase;
                    if u <= 1e-9 || u >= cycles - 1e-9 {
                        continue;
                    }
                    let p = phase + u;
                    if jump {
                        // A vertical jump: the level just before the edge, then just after.
                        points.push(at(u, y(p - 1e-9)));
                        points.push(at(u, y(p + 1e-9)));
                    } else {
                        points.push(at(u, y(p)));
                    }
                }
            }
            points.push(at(cycles, y(phase + cycles)));
            points.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9);
            polyline(&points, false)
        }
    };
    Ok(vec![shape("Wave", vec![path], false)])
}
