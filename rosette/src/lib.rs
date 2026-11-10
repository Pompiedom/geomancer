//! Rosette: a live effect that repeats an object's geometry around a centre point with rosette
//! (point-group) symmetry: rotations only (cyclic, Cₙ) or rotations plus mirrors (dihedral, Dₙ).
//! As a live effect the original stays editable and the rosette follows it.

use geomancer_common::{P, Value, centre, choice, flag, json, num, plugin, rect};
use std::f64::consts::TAU;

const MANIFEST: &str = r#"{
  "id": "io.github.pompiedom.geomancer.rosette",
  "name": "Rosette",
  "version": "0.2.0",
  "kind": "effect",
  "author": "Marco (Pompiedom)",
  "description": "Repeats the object around a centre point. Mirror adds a reflected copy between each pair (dihedral symmetry).",
  "params": {
    "copies": {"type": "int", "min": 1, "max": 360, "default": 6},
    "mirror": {"type": "bool", "default": false,
               "help": "Adds a reflected copy between each pair of copies, for mirror symmetry (dihedral) instead of rotation alone (cyclic)."},
    "centre": {"type": "choice", "options": ["bottom", "top", "left", "right", "middle"], "default": "bottom",
               "help": "The point of the object's bounds the copies turn around."},
    "offsetX": {"type": "number", "min": -10000, "max": 10000, "default": 0, "unit": "pt", "label": "Centre offset X"},
    "offsetY": {"type": "number", "min": -10000, "max": 10000, "default": 0, "unit": "pt", "label": "Centre offset Y"}
  }
}"#;

plugin!(MANIFEST, run);

fn run(input: &Value, params: &Value) -> Result<Value, String> {
    let n = num(params, "copies", 6.0).round().max(1.0) as usize;
    let mirror = flag(params, "mirror");
    let mut out = Vec::new();
    for obj in input["objects"].as_array().into_iter().flatten() {
        let subpaths = obj["path"]["subpaths"].as_array().cloned().unwrap_or_default();
        let Some(b) = rect(&obj["bounds"]).or_else(|| rect(&params["_context"]["bounds"])) else {
            out.push(json!({"path": {"subpaths": subpaths}}));
            continue;
        };
        let mid = centre(b);
        let pivot = match choice(params, "centre", "bottom") {
            "top" => (mid.0, b[1]),
            "left" => (b[0], mid.1),
            "right" => (b[2], mid.1),
            "middle" => mid,
            _ => (mid.0, b[3]),
        };
        let c = (pivot.0 + num(params, "offsetX", 0.0), pivot.1 + num(params, "offsetY", 0.0));
        let step = TAU / n as f64;
        // The mirror line runs from the centre through the object, turned half a step's half, so
        // the reflected copies land midway between the rotated ones.
        let along = if (mid.0 - c.0).hypot(mid.1 - c.1) > 1e-9 { (mid.1 - c.1).atan2(mid.0 - c.0) } else { -TAU / 4.0 };
        let axis = along + step / 4.0;
        let mut result = Vec::new();
        for k in 0..n {
            let a = step * k as f64;
            result.extend(subpaths.iter().map(|s| map_subpath(s, |p| rotate(p, c, a), false)));
        }
        if mirror {
            for k in 0..n {
                let a = step * k as f64;
                // Reflections reverse each subpath so the copies wind the same way as the
                // original: overlaps then stay filled under the non-zero rule.
                result.extend(subpaths.iter().map(|s| map_subpath(s, |p| rotate(reflect(p, c, axis), c, a), true)));
            }
        }
        out.push(json!({"path": {"subpaths": result}}));
    }
    Ok(json!({"objects": out}))
}

fn rotate(p: P, c: P, a: f64) -> P {
    let (s, co) = a.sin_cos();
    let (x, y) = (p.0 - c.0, p.1 - c.1);
    (c.0 + x * co - y * s, c.1 + x * s + y * co)
}

/// `p` mirrored in the line through `c` at angle `axis`.
fn reflect(p: P, c: P, axis: f64) -> P {
    let (s, co) = (2.0 * axis).sin_cos();
    let (x, y) = (p.0 - c.0, p.1 - c.1);
    (c.0 + x * co + y * s, c.1 + x * s - y * co)
}

fn point(v: &Value) -> Option<P> {
    Some((v.get(0)?.as_f64()?, v.get(1)?.as_f64()?))
}

/// A subpath with every anchor and handle moved by `f`; `reverse` also reverses its direction.
fn map_subpath(s: &Value, f: impl Fn(P) -> P, reverse: bool) -> Value {
    let mut anchors: Vec<Value> = s["anchors"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|a| {
            let mut a = a.clone();
            for key in ["p", "in", "out"] {
                if let Some(q) = point(&a[key]) {
                    let q = f(q);
                    a[key] = json!([q.0, q.1]);
                }
            }
            if reverse && let Some(o) = a.as_object_mut() {
                let (i, out) = (o.remove("in"), o.remove("out"));
                if let Some(i) = i {
                    o.insert("out".into(), i);
                }
                if let Some(out) = out {
                    o.insert("in".into(), out);
                }
            }
            a
        })
        .collect();
    if reverse {
        anchors.reverse();
    }
    let mut s = s.clone();
    s["anchors"] = json!(anchors);
    s
}
