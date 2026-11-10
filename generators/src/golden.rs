//! Golden Rectangle: a rectangle whose sides are in the golden ratio φ, cut into ever smaller
//! squares, with the golden spiral of quarter circles through them.

use crate::Shape;
use geomancer_common::{P, Value, choice, flag, json, num, placement, polyline, shape};

pub const SHAPE: Shape = Shape {
    id: "golden",
    label: "Golden Rectangle",
    params: r#"{
      "width": {"type": "number", "min": 1, "max": 10000, "default": 300, "unit": "pt",
                "help": "The long side; the short side is width / φ (φ ≈ 1.618)."},
      "orientation": {"type": "choice", "options": ["landscape", "portrait"], "default": "landscape"},
      "squares": {"type": "int", "min": 0, "max": 30, "default": 8,
                  "help": "How many squares to cut off, each the next smaller one."},
      "direction": {"type": "choice", "options": ["clockwise", "counterclockwise"], "default": "clockwise",
                    "help": "Which way the squares, and the spiral, wind inwards."},
      "spiral": {"type": "bool", "default": true, "help": "Adds the golden spiral as a path of its own."}
    }"#,
    run,
};

/// The Bézier handle length of a quarter circle, as a share of its radius.
const KAPPA: f64 = 0.552_284_749_830_793_4;

fn run(input: &Value, params: &Value) -> Result<Vec<Value>, String> {
    let phi = (1.0 + 5f64.sqrt()) / 2.0;
    let long = num(params, "width", 300.0);
    let (w, h) = if choice(params, "orientation", "landscape") == "portrait" { (long / phi, long) } else { (long, long / phi) };
    let c = placement(input, params);
    let n = num(params, "squares", 8.0).round().clamp(0.0, 30.0) as usize;
    // Counterclockwise mirrors the clockwise construction left to right.
    let ccw = choice(params, "direction", "clockwise") == "counterclockwise";
    let m = |p: P| -> P { if ccw { (2.0 * c.0 - p.0, p.1) } else { p } };

    let (mut x0, mut y0, mut x1, mut y1) = (c.0 - w / 2.0, c.1 - h / 2.0, c.0 + w / 2.0, c.1 + h / 2.0);
    let mut lines = vec![polyline(&[m((x0, y0)), m((x1, y0)), m((x1, y1)), m((x0, y1))], true)];
    // Each quarter circle: start, end and centre.
    let mut arcs: Vec<(P, P, P)> = Vec::new();
    // Cut the square off the long side, turning clockwise round the rectangle: left, top, right,
    // bottom, … A portrait rectangle starts with the top. Each arc ends where the next one starts.
    let first = if w >= h { 0 } else { 1 };
    for i in 0..n {
        match (first + i) % 4 {
            0 => {
                let s = y1 - y0;
                lines.push(polyline(&[m((x0 + s, y0)), m((x0 + s, y1))], false));
                arcs.push(((x0, y1), (x0 + s, y0), (x0 + s, y1)));
                x0 += s;
            }
            1 => {
                let s = x1 - x0;
                lines.push(polyline(&[m((x0, y0 + s)), m((x1, y0 + s))], false));
                arcs.push(((x0, y0), (x1, y0 + s), (x0, y0 + s)));
                y0 += s;
            }
            2 => {
                let s = y1 - y0;
                lines.push(polyline(&[m((x1 - s, y0)), m((x1 - s, y1))], false));
                arcs.push(((x1, y0), (x1 - s, y1), (x1 - s, y0)));
                x1 -= s;
            }
            _ => {
                let s = x1 - x0;
                lines.push(polyline(&[m((x0, y1 - s)), m((x1, y1 - s))], false));
                arcs.push(((x1, y1), (x0, y1 - s), (x1, y1 - s)));
                y1 -= s;
            }
        }
    }
    let mut shapes = vec![shape("Golden Rectangle", lines, false)];
    if flag(params, "spiral") && !arcs.is_empty() {
        // A quarter circle from a to b round o leaves a along o→b and arrives at b along a→o… so
        // its handles are a + κ(b − o) and b + κ(a − o).
        let pt = |p: P| json!([m(p).0, m(p).1]);
        let mut anchors = Vec::new();
        for (i, &(a, b, o)) in arcs.iter().enumerate() {
            let out = (a.0 + KAPPA * (b.0 - o.0), a.1 + KAPPA * (b.1 - o.1));
            if i == 0 {
                anchors.push(json!({"p": pt(a), "out": pt(out)}));
            } else if let Some(last) = anchors.last_mut() {
                last["out"] = pt(out);
                last["kind"] = json!("Smooth");
            }
            let inn = (b.0 + KAPPA * (a.0 - o.0), b.1 + KAPPA * (a.1 - o.1));
            anchors.push(json!({"p": pt(b), "in": pt(inn)}));
        }
        shapes.push(shape("Golden Spiral", vec![json!({"closed": false, "anchors": anchors})], false));
    }
    Ok(shapes)
}
