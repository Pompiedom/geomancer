//! Shared code for the Geomancer VectorCraft plug-ins (plug-in ABI v1, see VectorCraft's
//! `docs/plugins.md`): the ABI exports, reading parameters, turning parametric curves into
//! VectorCraft paths, and placing generated shapes.
//!
//! No imports and no WASI: everything runs inside VectorCraft's wasm sandbox.

pub use serde_json::{Value, json};

pub type P = (f64, f64);

/// Defines the five ABI exports (`memory` comes from the toolchain) for a plug-in whose manifest is
/// `$manifest` and whose work is done by `$run: fn(&Value, &Value) -> Result<Value, String>`.
/// `Err` becomes `{"error": …}`, which VectorCraft shows to the user.
#[macro_export]
macro_rules! plugin {
    ($manifest:expr, $run:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn vc_abi_version() -> u32 {
            1
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn vc_manifest() -> u64 {
            $crate::pack($manifest.as_bytes())
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn vc_alloc(size: u32) -> u32 {
            $crate::alloc(size)
        }

        /// # Safety
        /// The host guarantees both blocks came from `vc_alloc` and hold `*_len` bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn vc_run(input: *const u8, input_len: u32, params: *const u8, params_len: u32) -> i64 {
            unsafe { $crate::run(input, input_len, params, params_len, $run) }
        }
    };
}

pub fn alloc(size: u32) -> u32 {
    let mut block = Vec::<u8>::with_capacity(size as usize);
    let ptr = block.as_mut_ptr();
    std::mem::forget(block);
    ptr as u32
}

/// # Safety
/// `input` and `params` point at `input_len` and `params_len` readable bytes.
pub unsafe fn run(input: *const u8, input_len: u32, params: *const u8, params_len: u32, f: fn(&Value, &Value) -> Result<Value, String>) -> i64 {
    // SAFETY: see the function's contract.
    let (input, params) = unsafe { (std::slice::from_raw_parts(input, input_len as usize), std::slice::from_raw_parts(params, params_len as usize)) };
    let (Ok(input), Ok(params)) = (serde_json::from_slice::<Value>(input), serde_json::from_slice::<Value>(params)) else { return -1 };
    let out = f(&input, &params).unwrap_or_else(|e| json!({"error": e}));
    let Ok(bytes) = serde_json::to_vec(&out) else { return -2 };
    pack(bytes.leak()) as i64
}

pub fn pack(bytes: &[u8]) -> u64 {
    ((bytes.len() as u64) << 32) | bytes.as_ptr() as u64
}

pub fn num(params: &Value, key: &str, default: f64) -> f64 {
    params[key].as_f64().filter(|x| x.is_finite()).unwrap_or(default)
}

pub fn flag(params: &Value, key: &str) -> bool {
    params[key].as_bool().unwrap_or(false)
}

pub fn choice<'a>(params: &'a Value, key: &str, default: &'a str) -> &'a str {
    params[key].as_str().unwrap_or(default)
}

/// `[x0, y0, x1, y1]` read from a JSON array.
pub fn rect(v: &Value) -> Option<[f64; 4]> {
    let a = v.as_array()?;
    if a.len() != 4 {
        return None;
    }
    let mut r = [0.0; 4];
    for (i, x) in a.iter().enumerate() {
        r[i] = x.as_f64()?;
    }
    Some(r)
}

pub fn centre(r: [f64; 4]) -> P {
    ((r[0] + r[2]) / 2.0, (r[1] + r[3]) / 2.0)
}

/// The cubic Bézier subpath that follows `f` from `t0` to `t1` in `segments` equal steps: each
/// anchor lies on the curve and its handles follow the curve's tangent (cubic Hermite
/// interpolation). For a closed curve `f(t1)` must equal `f(t0)`; that duplicate anchor is dropped.
pub fn curve(f: impl Fn(f64) -> P, t0: f64, t1: f64, segments: usize, closed: bool) -> Value {
    let n = segments.max(1);
    let dt = (t1 - t0) / n as f64;
    let h = dt * 1e-4;
    // Tangent scaled to one step, by central differences (one-sided at the ends of an open curve).
    let tangent = |t: f64| {
        let (a, b) = if closed { (t - h, t + h) } else { ((t - h).max(t0.min(t1)), (t + h).min(t0.max(t1))) };
        let (pa, pb) = (f(a), f(b));
        let k = dt / (b - a) / 3.0;
        ((pb.0 - pa.0) * k, (pb.1 - pa.1) * k)
    };
    let count = if closed { n } else { n + 1 };
    let anchors: Vec<Value> = (0..count)
        .map(|i| {
            let t = if i == n { t1 } else { t0 + dt * i as f64 };
            let p = f(t);
            let d = tangent(t);
            let mut a = json!({"p": [p.0, p.1]});
            if closed || i > 0 {
                a["in"] = json!([p.0 - d.0, p.1 - d.1]);
            }
            if closed || i < n {
                a["out"] = json!([p.0 + d.0, p.1 + d.1]);
            }
            if closed || (i > 0 && i < n) {
                a["kind"] = json!("Smooth");
            }
            a
        })
        .collect();
    json!({"closed": closed, "anchors": anchors})
}

/// Where a generator puts its shape: the centre of the selection, else of the artboard.
pub fn placement(input: &Value, params: &Value) -> P {
    let ctx = &params["_context"];
    let selected = input["objects"].as_array().is_some_and(|o| !o.is_empty());
    let r = if selected { rect(&ctx["bounds"]) } else { None }.or_else(|| rect(&ctx["artboard"]));
    r.map(centre).unwrap_or((0.0, 0.0))
}

/// An open or closed subpath of corner anchors through `points`.
pub fn polyline(points: &[P], closed: bool) -> Value {
    let anchors: Vec<Value> = points.iter().map(|p| json!({"p": [p.0, p.1]})).collect();
    json!({"closed": closed, "anchors": anchors})
}

/// A new path named `name`: a 1 pt black stroke, or with `fill` a black fill and no stroke.
pub fn shape(name: &str, subpaths: Vec<Value>, fill: bool) -> Value {
    let black = json!({"type": "solid", "color": {"model": "gray", "k": 1}});
    let mut shape = json!({
        "type": "path",
        "name": name,
        "path": {"subpaths": subpaths},
        "fillRule": "nonzero",
        "strokes": [{"paint": black, "width": 1}],
    });
    if fill {
        shape["fills"] = json!([black]);
        shape["strokes"] = json!([]);
    }
    shape
}

/// A generator's output: every selected object kept as it is, and the new shapes above them.
pub fn generated(input: &Value, shapes: Vec<Value>) -> Value {
    let mut objects: Vec<Value> = input["objects"].as_array().into_iter().flatten().filter_map(|o| o.get("id")).map(|id| json!({"id": id})).collect();
    objects.extend(shapes);
    json!({"objects": objects})
}
