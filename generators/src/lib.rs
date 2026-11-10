//! Geomancer: one object filter that generates geometric shapes (spiral, rose curve, wave, golden
//! rectangle) at the centre of the selection, or of the artboard when nothing is selected.
//!
//! In VectorCraft it is a submenu, Object › Plug-ins › Geomancer: each shape is a command with a
//! dialog of its own, and the plug-in's own item, Geomancer…, opens one dialog with a shape picker
//! that shows the chosen shape's fields (each prefixed with the shape's id, such as `rose_n`).

use geomancer_common::{Value, json, plugin};
use std::sync::OnceLock;

mod golden;
mod rose;
mod spiral;
mod wave;

/// A shape: its command id, its label, its parameters (a JSON object in dialog order) and the
/// function that draws it, which returns the new objects.
pub struct Shape {
    id: &'static str,
    label: &'static str,
    params: &'static str,
    run: fn(&Value, &Value) -> Result<Vec<Value>, String>,
}

const SHAPES: [Shape; 4] = [spiral::SHAPE, rose::SHAPE, wave::SHAPE, golden::SHAPE];

fn manifest() -> &'static str {
    static MANIFEST: OnceLock<String> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        let labels: Vec<&str> = SHAPES.iter().map(|s| s.label).collect();
        let mut own = serde_json::Map::new();
        own.insert("shape".into(), json!({"type": "choice", "options": labels, "default": labels[0], "label": "Shape"}));
        let mut commands = Vec::new();
        for s in &SHAPES {
            let params: serde_json::Map<String, Value> = serde_json::from_str(s.params).unwrap_or_default();
            for (k, spec) in &params {
                let mut spec = spec.clone();
                spec["visibleWhen"] = json!({"shape": [s.label]});
                own.insert(format!("{}_{k}", s.id), spec);
            }
            commands.push(json!({"id": s.id, "label": s.label, "params": params}));
        }
        json!({
            "id": "io.github.pompiedom.geomancer",
            "name": "Geomancer",
            "version": env!("CARGO_PKG_VERSION"),
            "kind": "filter",
            "author": "Marco (Pompiedom)",
            "description": "Geometric shapes: spiral, rose curve, wave and golden rectangle, placed at the centre of the selection, or of the artboard when nothing is selected.",
            "params": own,
            "commands": commands,
        })
        .to_string()
    })
}

plugin!(manifest(), run);

fn run(input: &Value, params: &Value) -> Result<Value, String> {
    let ctx = &params["_context"];
    let shapes = if ctx["mode"] == "command" {
        let id = ctx["command"].as_str().unwrap_or_default();
        let s = SHAPES.iter().find(|s| s.id == id).ok_or_else(|| format!("Unknown command {id:?}."))?;
        (s.run)(input, params)?
    } else {
        // The own dialog: the chosen shape, its parameters without their prefix.
        let label = params["shape"].as_str().unwrap_or(SHAPES[0].label);
        let s = SHAPES.iter().find(|s| s.label == label).ok_or_else(|| format!("Unknown shape {label:?}."))?;
        let prefix = format!("{}_", s.id);
        let mut own = serde_json::Map::new();
        for (k, v) in params.as_object().into_iter().flatten() {
            if let Some(name) = k.strip_prefix(&prefix) {
                own.insert(name.into(), v.clone());
            }
        }
        own.insert("_context".into(), ctx.clone());
        (s.run)(input, &Value::Object(own))?
    };
    Ok(geomancer_common::generated(input, shapes))
}
