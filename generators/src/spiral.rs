//! Spiral: an Archimedean spiral, or a two-armed Fermat spiral (an S through the centre).

use crate::Shape;
use geomancer_common::{P, Value, choice, curve, flag, num, placement, shape};
use std::f64::consts::TAU;

pub const SHAPE: Shape = Shape {
    id: "spiral",
    label: "Spiral",
    params: r#"{
      "innerRadius": {"type": "number", "min": 0, "max": 5000, "default": 0, "unit": "pt", "label": "Inner radius"},
      "outerRadius": {"type": "number", "min": 1, "max": 5000, "default": 100, "unit": "pt", "label": "Outer radius"},
      "turns": {"type": "number", "min": 0.25, "max": 100, "default": 5},
      "fermat": {"type": "bool", "default": false, "label": "Fermat",
                 "help": "A two-armed Fermat spiral (r = ±√θ), an S through the centre, instead of an Archimedean one. With an inner radius the S is cut out of the middle."},
      "direction": {"type": "choice", "options": ["clockwise", "counterclockwise"], "default": "clockwise"},
      "startAngle": {"type": "number", "min": -360, "max": 360, "default": 0, "unit": "°", "label": "Start angle"}
    }"#,
    run,
};

fn run(input: &Value, params: &Value) -> Result<Vec<Value>, String> {
    let r_in = num(params, "innerRadius", 0.0).max(0.0);
    let r_out = num(params, "outerRadius", 100.0);
    let turns = num(params, "turns", 5.0);
    if r_out <= r_in {
        return Err("Outer radius must be larger than inner radius.".into());
    }
    let span = TAU * turns;
    // y points down, so a growing angle turns clockwise on screen.
    let dir = if choice(params, "direction", "clockwise") == "counterclockwise" { -1.0 } else { 1.0 };
    let start = num(params, "startAngle", 0.0).to_radians();
    let c = placement(input, params);
    let at = move |r: f64, theta: f64| -> P {
        let a = start + dir * theta;
        (c.0 + r * a.cos(), c.1 + r * a.sin())
    };

    let paths = if flag(params, "fermat") {
        // Fermat's spiral r = ±√θ: two interleaved arms, the negative radius drawing the second
        // one, mirrored through the centre. Parametrised by the signed radius so the arms run
        // through the centre as one S-shaped curve. Each step sweeps up to 2·span/segments, so 24
        // a turn per arm keeps steps under 30°. With an inner radius the S is cut out of the
        // middle, leaving the two arms apart.
        let (a2, d2) = (r_in * r_in, r_out * r_out - r_in * r_in);
        let theta = move |r: f64| span * (r * r - a2) / d2;
        let arm = ((24.0 * turns).ceil() as usize).max(8);
        if r_in == 0.0 {
            vec![curve(|r| at(r, theta(r)), -r_out, r_out, 2 * arm, false)]
        } else {
            vec![curve(|r| at(r, theta(r)), -r_out, -r_in, arm, false), curve(|r| at(r, theta(r)), r_in, r_out, arm, false)]
        }
    } else {
        // Archimedean: r grows linearly with the angle; 12 segments a turn.
        let segments = ((12.0 * turns).ceil() as usize).max(4);
        vec![curve(|t| at(r_in + (r_out - r_in) * t / span, t), 0.0, span, segments, false)]
    };
    Ok(vec![shape("Spiral", paths, false)])
}
