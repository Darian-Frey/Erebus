// CPU-side colour-stop representation for the nebula gradient LUT. Lives
// in a preset, edited via the UI (Phase 8 polish), uploaded to the GPU
// at startup and on preset load.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GradientStop {
    pub position: f32,
    pub color: [f32; 3],
}

/// Default synthwave palette used at startup until a preset is loaded.
pub fn synthwave_default() -> Vec<GradientStop> {
    vec![
        GradientStop {
            position: 0.00,
            color: [0.00, 0.00, 0.00],
        },
        GradientStop {
            position: 0.10,
            color: [0.04, 0.00, 0.10],
        },
        GradientStop {
            position: 0.30,
            color: [0.30, 0.05, 0.45],
        },
        GradientStop {
            position: 0.55,
            color: [0.85, 0.20, 0.65],
        },
        GradientStop {
            position: 0.78,
            color: [0.40, 0.55, 1.20],
        },
        GradientStop {
            position: 1.00,
            color: [1.20, 1.10, 1.40],
        },
    ]
}

/// Linearly sample the colour at `t` ∈ [0, 1].
pub fn sample(stops: &[GradientStop], t: f32) -> [f32; 3] {
    if stops.is_empty() {
        return [0.0; 3];
    }
    if t <= stops[0].position {
        return stops[0].color;
    }
    if t >= stops[stops.len() - 1].position {
        return stops[stops.len() - 1].color;
    }
    for i in 0..stops.len() - 1 {
        let a = stops[i];
        let b = stops[i + 1];
        if t >= a.position && t <= b.position {
            let span = b.position - a.position;
            let k = if span > 0.0 {
                (t - a.position) / span
            } else {
                0.0
            };
            return [
                a.color[0] + (b.color[0] - a.color[0]) * k,
                a.color[1] + (b.color[1] - a.color[1]) * k,
                a.color[2] + (b.color[2] - a.color[2]) * k,
            ];
        }
    }
    [0.0; 3]
}
