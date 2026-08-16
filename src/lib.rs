//! Agent-native scene + physics inspect + headless PNG (increment 1).

mod physics;
mod render;
mod scene;

pub use physics::{step_physics, PhysicsBodyState, PhysicsContact, PhysicsDump};
pub use render::{render_scene, render_scene_to_png, FRAME_HEIGHT, FRAME_WIDTH};
pub use scene::{demo_scene, demo_scene_json, parse_scene, Body, Camera, Light, Material, Scene, Shape};

use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_STEPS: u32 = 90;
pub const DEFAULT_DT: f32 = 1.0 / 60.0;

#[derive(Debug, Clone)]
pub struct ArtifactPaths {
    pub scene: PathBuf,
    pub physics: PathBuf,
    pub frame: PathBuf,
}

/// Write scene JSON, step physics, write physics dump, render post-step PNG.
pub fn run_increment1(out_dir: &Path, steps: u32, dt: f32, width: u32, height: u32) -> Result<ArtifactPaths, String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir:?}: {e}"))?;

    let scene = demo_scene();
    let scene_path = out_dir.join("scene.json");
    let physics_path = out_dir.join("physics.json");
    let frame_path = out_dir.join("frame.png");

    let json = serde_json::to_string_pretty(&scene).map_err(|e| e.to_string())?;
    fs::write(&scene_path, json).map_err(|e| format!("write scene: {e}"))?;

    let dump = step_physics(&scene, steps, dt)?;
    let dump_json = serde_json::to_string_pretty(&dump).map_err(|e| e.to_string())?;
    fs::write(&physics_path, dump_json).map_err(|e| format!("write physics: {e}"))?;

    let mut framed = scene.clone();
    apply_physics_to_scene(&mut framed, &dump);
    render_scene_to_png(&framed, width, height, &frame_path)?;

    Ok(ArtifactPaths {
        scene: scene_path,
        physics: physics_path,
        frame: frame_path,
    })
}

pub fn apply_physics_to_scene(scene: &mut Scene, dump: &PhysicsDump) {
    for body in &mut scene.bodies {
        if let Some(state) = dump.bodies.iter().find(|b| b.id == body.id) {
            body.position = state.position;
            body.rotation_wxyz = state.rotation_wxyz;
        }
    }
}
