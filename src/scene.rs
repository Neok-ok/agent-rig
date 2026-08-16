use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mesh::{load_obj, resolve_mesh_path, resolve_texture_path, TriangleMesh};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub bodies: Vec<Body>,
    #[serde(skip, default)]
    pub mesh_search_dirs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub position: [f32; 3],
    pub look_at: [f32; 3],
    pub fov_y_deg: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Light {
    #[serde(rename = "directional")]
    Directional {
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub id: String,
    pub shape: Shape,
    pub position: [f32; 3],
    #[serde(default = "identity_wxyz")]
    pub rotation_wxyz: [f32; 4],
    pub mass: f32,
    #[serde(default)]
    pub linear_velocity: [f32; 3],
    pub material: Material,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Shape {
    #[serde(rename = "box")]
    Box { size: [f32; 3] },
    #[serde(rename = "sphere")]
    Sphere { radius: f32 },
    #[serde(rename = "mesh")]
    Mesh {
        path: String,
        #[serde(default = "default_mesh_collider")]
        collider: MeshCollider,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshCollider {
    ConvexHull,
    Trimesh,
}

fn default_mesh_collider() -> MeshCollider {
    MeshCollider::ConvexHull
}

impl Shape {
    pub fn collider_kind(&self) -> &'static str {
        match self {
            Shape::Box { .. } => "cuboid",
            Shape::Sphere { .. } => "ball",
            Shape::Mesh {
                collider: MeshCollider::ConvexHull,
                ..
            } => "convex_hull",
            Shape::Mesh {
                collider: MeshCollider::Trimesh,
                ..
            } => "trimesh",
        }
    }
}

impl Scene {
    pub fn resolve_mesh(&self, path: &str) -> Result<PathBuf, String> {
        resolve_mesh_path(path, &self.mesh_search_dirs)
    }

    pub fn load_body_mesh(&self, path: &str) -> Result<TriangleMesh, String> {
        load_obj(&self.resolve_mesh(path)?)
    }

    pub fn resolve_texture(&self, path: &str) -> Result<PathBuf, String> {
        resolve_texture_path(path, &self.mesh_search_dirs)
    }

    pub fn with_default_mesh_search(mut self) -> Self {
        self.mesh_search_dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        self
    }
}

pub fn load_scene_from_path(path: &Path) -> Result<Scene, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("read scene {path:?}: {e}"))?;
    let mut scene = parse_scene(&txt)?;
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            scene.mesh_search_dirs.push(dir.to_path_buf());
        }
    }
    scene.mesh_search_dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    Ok(scene)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub albedo: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub albedo_map: Option<String>,
}

fn identity_wxyz() -> [f32; 4] {
    [1.0, 0.0, 0.0, 0.0]
}

pub const DEMO_SCENE_JSON: &str = r#"{
  "camera": { "position": [4.2, 2.4, 5.2], "look_at": [0.0, 0.35, 0.0], "fov_y_deg": 40 },
  "lights": [{ "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 }],
  "bodies": [
    {
      "id": "ground",
      "shape": { "type": "box", "size": [8, 0.2, 8] },
      "position": [0, -0.1, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.35, 0.36, 0.38], "roughness": 0.8, "metallic": 0.0 }
    },
    {
      "id": "ball",
      "shape": { "type": "sphere", "radius": 0.4 },
      "position": [0, 2.2, 0],
      "mass": 1.0,
      "material": { "albedo": [0.85, 0.22, 0.16], "roughness": 0.35, "metallic": 0.05 }
    }
  ]
}"#;

pub fn demo_scene_json() -> &'static str {
    DEMO_SCENE_JSON
}

pub fn parse_scene(json: &str) -> Result<Scene, String> {
    serde_json::from_str(json).map_err(|e| format!("parse scene: {e}"))
}

pub fn demo_scene() -> Scene {
    parse_scene(DEMO_SCENE_JSON).expect("demo scene JSON is valid")
}

pub const INCREMENT2_SCENE_JSON: &str = r#"{
  "camera": { "position": [5.2, 2.8, 6.0], "look_at": [0.2, 0.4, 0.0], "fov_y_deg": 40 },
  "lights": [{ "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 }],
  "bodies": [
    {
      "id": "ground",
      "shape": { "type": "box", "size": [10, 0.2, 10] },
      "position": [0, -0.1, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.35, 0.36, 0.38], "roughness": 0.8, "metallic": 0.0 }
    },
    {
      "id": "ball",
      "shape": { "type": "sphere", "radius": 0.35 },
      "position": [-1.6, 0.9, 0],
      "mass": 1.0,
      "linear_velocity": [4.5, 0, 0],
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9 }
    },
    {
      "id": "crate",
      "shape": { "type": "box", "size": [0.7, 0.7, 0.7] },
      "position": [0.4, 0.35, 0],
      "mass": 1.0,
      "material": { "albedo": [0.22, 0.38, 0.28], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "stopper",
      "shape": { "type": "box", "size": [0.4, 0.6, 0.4] },
      "position": [1.6, 0.3, 0],
      "mass": 0,
      "material": { "albedo": [0.7, 0.72, 0.75], "roughness": 0.2, "metallic": 0.85 }
    }
  ]
}"#;

pub fn increment2_scene_json() -> &'static str {
    INCREMENT2_SCENE_JSON
}

pub fn increment2_scene() -> Scene {
    parse_scene(INCREMENT2_SCENE_JSON).expect("increment2 scene JSON is valid")
}

pub const INCREMENT3_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.4, 2.8, 7.4], "look_at": [0.5, 0.7, 0.0], "fov_y_deg": 40 },
  "lights": [{ "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 }],
  "bodies": [
    {
      "id": "ground",
      "shape": { "type": "box", "size": [14, 0.2, 8] },
      "position": [0, -0.1, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.35, 0.36, 0.38], "roughness": 0.8, "metallic": 0.0 }
    },
    {
      "id": "ramp",
      "shape": { "type": "box", "size": [4.2, 0.2, 1.8] },
      "position": [0, 0.91, 0],
      "rotation_wxyz": [0.9659258, 0, 0, -0.258819],
      "mass": 0,
      "material": { "albedo": [0.55, 0.42, 0.28], "roughness": 0.7, "metallic": 0.0 }
    },
    {
      "id": "ball",
      "shape": { "type": "sphere", "radius": 0.32 },
      "position": [-1.09, 2.03, 0],
      "mass": 1.0,
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9 }
    },
    {
      "id": "crate",
      "shape": { "type": "box", "size": [0.56, 0.56, 0.56] },
      "position": [2.55, 0.28, 0],
      "mass": 0.8,
      "material": { "albedo": [0.22, 0.38, 0.28], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "stopper",
      "shape": { "type": "box", "size": [0.28, 0.55, 1.4] },
      "position": [3.55, 0.275, 0],
      "mass": 0,
      "material": { "albedo": [0.7, 0.72, 0.75], "roughness": 0.2, "metallic": 0.85 }
    }
  ]
}"#;

pub fn increment3_scene_json() -> &'static str {
    INCREMENT3_SCENE_JSON
}

pub fn increment3_scene() -> Scene {
    parse_scene(INCREMENT3_SCENE_JSON).expect("increment3 scene JSON is valid")
}

pub const INCREMENT4_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.5, 2.15, 5.1], "look_at": [0.15, 0.42, 0.0], "fov_y_deg": 40 },
  "lights": [{ "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 }],
  "bodies": [
    {
      "id": "ground",
      "shape": { "type": "box", "size": [10, 0.2, 10] },
      "position": [0, -0.1, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.35, 0.36, 0.38], "roughness": 0.8, "metallic": 0.0 }
    },
    {
      "id": "rock",
      "shape": { "type": "mesh", "path": "meshes/rock.obj", "collider": "convex_hull" },
      "position": [0.45, 0.002, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 12.0,
      "material": { "albedo": [0.48, 0.52, 0.62], "roughness": 0.28, "metallic": 0.2 }
    },
    {
      "id": "ball",
      "shape": { "type": "sphere", "radius": 0.32 },
      "position": [-1.35, 0.85, 0.04],
      "mass": 1.0,
      "linear_velocity": [3.4, 0.15, 0],
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9 }
    },
    {
      "id": "crate",
      "shape": { "type": "box", "size": [0.44, 0.44, 0.44] },
      "position": [1.55, 0.22, -0.55],
      "mass": 0.6,
      "material": { "albedo": [0.22, 0.38, 0.28], "roughness": 0.85, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment4_scene_json() -> &'static str {
    INCREMENT4_SCENE_JSON
}

pub fn increment4_scene() -> Scene {
    parse_scene(INCREMENT4_SCENE_JSON)
        .expect("increment4 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT5_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.5, 2.15, 5.1], "look_at": [0.15, 0.42, 0.0], "fov_y_deg": 40 },
  "lights": [{ "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 }],
  "bodies": [
    {
      "id": "ground",
      "shape": { "type": "box", "size": [10, 0.2, 10] },
      "position": [0, -0.1, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.35, 0.36, 0.38], "roughness": 0.8, "metallic": 0.0 }
    },
    {
      "id": "rock",
      "shape": { "type": "mesh", "path": "meshes/rock.obj", "collider": "convex_hull" },
      "position": [0.45, 0.002, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 12.0,
      "material": { "albedo": [0.48, 0.52, 0.62], "roughness": 0.28, "metallic": 0.2, "albedo_map": "textures/rock.png" }
    },
    {
      "id": "ball",
      "shape": { "type": "sphere", "radius": 0.32 },
      "position": [-1.35, 0.85, 0.04],
      "mass": 1.0,
      "linear_velocity": [3.4, 0.15, 0],
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9 }
    },
    {
      "id": "crate",
      "shape": { "type": "box", "size": [0.44, 0.44, 0.44] },
      "position": [1.55, 0.22, -0.55],
      "mass": 0.6,
      "material": { "albedo": [0.22, 0.38, 0.28], "roughness": 0.85, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment5_scene_json() -> &'static str {
    INCREMENT5_SCENE_JSON
}

pub fn increment5_scene() -> Scene {
    parse_scene(INCREMENT5_SCENE_JSON)
        .expect("increment5 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT6_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.5, 2.15, 5.1], "look_at": [0.15, 0.42, 0.0], "fov_y_deg": 40 },
  "lights": [{ "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 }],
  "bodies": [
    {
      "id": "ground",
      "shape": { "type": "box", "size": [10, 0.2, 10] },
      "position": [0, -0.1, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.35, 0.36, 0.38], "roughness": 0.8, "metallic": 0.0 }
    },
    {
      "id": "rock",
      "shape": { "type": "mesh", "path": "meshes/rock.obj", "collider": "convex_hull" },
      "position": [0.45, 0.002, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 12.0,
      "material": { "albedo": [0.48, 0.52, 0.62], "roughness": 0.28, "metallic": 0.2, "albedo_map": "textures/rock.png" }
    },
    {
      "id": "wedge",
      "shape": { "type": "mesh", "path": "meshes/wedge.obj", "collider": "trimesh" },
      "position": [1.72, 0.0, 0.38],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.62, 0.38, 0.18], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "ball",
      "shape": { "type": "sphere", "radius": 0.32 },
      "position": [-1.35, 0.85, 0.04],
      "mass": 1.0,
      "linear_velocity": [3.4, 0.15, 0],
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9 }
    },
    {
      "id": "crate",
      "shape": { "type": "box", "size": [0.44, 0.44, 0.44] },
      "position": [1.55, 0.22, -0.70],
      "mass": 0.6,
      "material": { "albedo": [0.22, 0.38, 0.28], "roughness": 0.85, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment6_scene_json() -> &'static str {
    INCREMENT6_SCENE_JSON
}

pub fn increment6_scene() -> Scene {
    parse_scene(INCREMENT6_SCENE_JSON)
        .expect("increment6 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT7_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [{ "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 }],
  "bodies": [
    {
      "id": "ground",
      "shape": { "type": "mesh", "path": "meshes/bowl.obj", "collider": "trimesh" },
      "position": [0, 0, 0],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.48, 0.44, 0.38], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "rock",
      "shape": { "type": "mesh", "path": "meshes/rock.obj", "collider": "convex_hull" },
      "position": [0.40, 0.002, 0.08],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 12.0,
      "material": { "albedo": [0.48, 0.52, 0.62], "roughness": 0.28, "metallic": 0.2, "albedo_map": "textures/rock.png" }
    },
    {
      "id": "ball",
      "shape": { "type": "sphere", "radius": 0.32 },
      "position": [-1.10, 0.36, 0.10],
      "mass": 1.0,
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9 }
    }
  ]
}"#;

pub fn increment7_scene_json() -> &'static str {
    INCREMENT7_SCENE_JSON
}

pub fn increment7_scene() -> Scene {
    parse_scene(INCREMENT7_SCENE_JSON)
        .expect("increment7 scene JSON is valid")
        .with_default_mesh_search()
}
