use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mesh::{load_mesh, resolve_mesh_path, resolve_texture_path, TriangleMesh};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub bodies: Vec<Body>,
    /// Authorable joints. `anchor` is world-space; converted to local on each body
    /// at spawn. `axis` is a world-space direction (horizontal so gravity hangs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joints: Vec<Joint>,
    #[serde(skip, default)]
    pub mesh_search_dirs: Vec<PathBuf>,
}

/// Scene-authored constraint. Internally tagged with `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Joint {
    /// Revolute / hinge. `anchor` is world-space; `axis` is world-space.
    #[serde(rename = "hinge")]
    Hinge {
        body_a: String,
        body_b: String,
        anchor: [f32; 3],
        axis: [f32; 3],
    },
    /// Prismatic / slider. `axis` is world-space; `limits` are [min, max]
    /// along that axis from the closed pose. Optional `anchor` is the
    /// closed-pose world attachment (defaults to body_b origin).
    #[serde(rename = "slider")]
    Slider {
        body_a: String,
        body_b: String,
        axis: [f32; 3],
        limits: [f32; 2],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<[f32; 3]>,
    },
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
    #[serde(rename = "point")]
    Point {
        position: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    },
    #[serde(rename = "area")]
    Area {
        position: [f32; 3],
        /// World-space rectangle [width, height]. Softness comes from this size.
        size: [f32; 2],
        color: [f32; 3],
        intensity: f32,
        #[serde(default = "default_area_normal")]
        normal: [f32; 3],
    },
}

fn default_area_normal() -> [f32; 3] {
    [0.0, -1.0, 0.0]
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
        load_mesh(&self.resolve_mesh(path)?)
    }

    pub fn resolve_texture(&self, path: &str) -> Result<PathBuf, String> {
        resolve_texture_path(path, &self.mesh_search_dirs)
    }

    pub fn with_default_mesh_search(mut self) -> Self {
        self.mesh_search_dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        self
    }

    /// Material used for shading: glTF pbrMetallicRoughness when present, else scene JSON.
    pub fn resolved_body_material(&self, body: &Body) -> Result<Material, String> {
        if let Shape::Mesh { path, .. } = &body.shape {
            let mesh = self.load_body_mesh(path)?;
            if let Some(gm) = &mesh.gltf_material {
                return Ok(Material {
                    albedo: gm.base_color_rgb(),
                    roughness: gm.roughness_factor,
                    metallic: gm.metallic_factor,
                    albedo_map: gm.base_color_texture_path.as_ref().map(|p| {
                        p.to_string_lossy().into_owned()
                    }),
                    clearcoat: 0.0,
                    clearcoat_roughness: 0.0,
                    sheen: 0.0,
                    sheen_roughness: 0.5,
                    sheen_color: [1.0, 1.0, 1.0],
                    anisotropy: 0.0,
                    anisotropy_rotation: 0.0,
                    iridescence: 0.0,
                    iridescence_ior: 1.3,
                    iridescence_thickness: 400.0,
                    dispersion: gm.dispersion,
                });
            }
        }
        Ok(body.material.clone())
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
    /// Extra dielectric coat (0 = off). Optional; serde default 0.0.
    #[serde(default)]
    pub clearcoat: f32,
    /// Coat microfacet roughness. Softness is this authored value, not a hidden constant.
    #[serde(default)]
    pub clearcoat_roughness: f32,
    /// Extra fabric/velvet sheen (0 = off). Optional; serde default 0.0.
    #[serde(default)]
    pub sheen: f32,
    /// Sheen microfacet roughness. Softness is this authored value, not a hidden constant.
    #[serde(default = "default_sheen_roughness")]
    pub sheen_roughness: f32,
    /// Sheen tint. Default white.
    #[serde(default = "default_sheen_color")]
    pub sheen_color: [f32; 3],
    /// Brushed-metal anisotropy (0 = isotropic). Optional; serde default 0.0.
    /// Strength of the GGX stretch is this authored value, not a hidden constant.
    #[serde(default)]
    pub anisotropy: f32,
    /// Tangent-frame rotation in radians. Direction is this authored value.
    #[serde(default)]
    pub anisotropy_rotation: f32,
    /// Thin-film iridescence (0 = off). Optional; serde default 0.0.
    /// Mix of the rainbow Fresnel is this authored value, not a hidden constant.
    #[serde(default)]
    pub iridescence: f32,
    /// Film IOR. Optical path 2*n*d*cos(θ) uses this authored n.
    #[serde(default = "default_iridescence_ior")]
    pub iridescence_ior: f32,
    /// Film thickness in nanometres. Optical path uses this authored d.
    #[serde(default = "default_iridescence_thickness")]
    pub iridescence_thickness: f32,
    /// KHR_materials_dispersion factor (20/Abbe). 0 = no chromatic split.
    /// Strength of the R/G/B IOR offset is this authored value, not a hidden constant.
    #[serde(default)]
    pub dispersion: f32,
}

fn identity_wxyz() -> [f32; 4] {
    [1.0, 0.0, 0.0, 0.0]
}

fn default_sheen_roughness() -> f32 {
    0.5
}

fn default_sheen_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_iridescence_ior() -> f32 {
    1.3
}

fn default_iridescence_thickness() -> f32 {
    400.0
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

pub const INCREMENT8_SCENE_JSON: &str = r#"{
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.74, 0.64, 0.50], "roughness": 0.76, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment8_scene_json() -> &'static str {
    INCREMENT8_SCENE_JSON
}

pub fn increment8_scene() -> Scene {
    parse_scene(INCREMENT8_SCENE_JSON)
        .expect("increment8 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT9_SCENE_JSON: &str = r#"{
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment9_scene_json() -> &'static str {
    INCREMENT9_SCENE_JSON
}

pub fn increment9_scene() -> Scene {
    parse_scene(INCREMENT9_SCENE_JSON)
        .expect("increment9 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT10_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "point", "position": [1.00, 0.78, 0.85], "color": [1.0, 0.75, 0.45], "intensity": 14.0 }
  ],
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment10_scene_json() -> &'static str {
    INCREMENT10_SCENE_JSON
}

pub fn increment10_scene() -> Scene {
    parse_scene(INCREMENT10_SCENE_JSON)
        .expect("increment10 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT11_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "point", "position": [0.55, 0.82, 1.10], "color": [1.0, 0.75, 0.45], "intensity": 14.0 }
  ],
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment11_scene_json() -> &'static str {
    INCREMENT11_SCENE_JSON
}

pub fn increment11_scene() -> Scene {
    parse_scene(INCREMENT11_SCENE_JSON)
        .expect("increment11 scene JSON is valid")
        .with_default_mesh_search()
}

pub fn increment12_scene_json() -> &'static str {
    INCREMENT11_SCENE_JSON
}

pub fn increment12_scene() -> Scene {
    increment11_scene()
}

pub fn increment13_scene_json() -> &'static str {
    INCREMENT11_SCENE_JSON
}

pub fn increment13_scene() -> Scene {
    increment11_scene()
}

pub fn increment14_scene_json() -> &'static str {
    INCREMENT11_SCENE_JSON
}

pub fn increment14_scene() -> Scene {
    increment11_scene()
}

pub const INCREMENT15_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "point", "position": [0.55, 0.82, 1.10], "color": [1.0, 0.75, 0.45], "intensity": 14.0 }
  ],
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
      "linear_velocity": [3.4, 0.15, 0.45],
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment15_scene_json() -> &'static str {
    INCREMENT15_SCENE_JSON
}

pub fn increment15_scene() -> Scene {
    parse_scene(INCREMENT15_SCENE_JSON)
        .expect("increment15 scene JSON is valid")
        .with_default_mesh_search()
}

pub fn increment16_scene_json() -> &'static str {
    INCREMENT11_SCENE_JSON
}

pub fn increment16_scene() -> Scene {
    increment11_scene()
}

pub const INCREMENT17_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "point", "position": [0.55, 0.82, 1.10], "color": [1.0, 0.75, 0.45], "intensity": 14.0 }
  ],
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment17_scene_json() -> &'static str {
    INCREMENT17_SCENE_JSON
}

pub fn increment17_scene() -> Scene {
    parse_scene(INCREMENT17_SCENE_JSON)
        .expect("increment17 scene JSON is valid")
        .with_default_mesh_search()
}

pub fn increment18_scene_json() -> &'static str {
    INCREMENT18_SCENE_JSON
}

pub fn increment18_scene() -> Scene {
    parse_scene(INCREMENT18_SCENE_JSON)
        .expect("increment18 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT18_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment19_scene_json() -> &'static str {
    INCREMENT18_SCENE_JSON
}

pub fn increment19_scene() -> Scene {
    increment18_scene()
}

pub const INCREMENT20_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment20_scene_json() -> &'static str {
    INCREMENT20_SCENE_JSON
}

pub fn increment20_scene() -> Scene {
    parse_scene(INCREMENT20_SCENE_JSON)
        .expect("increment20 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT21_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment21_scene_json() -> &'static str {
    INCREMENT21_SCENE_JSON
}

pub fn increment21_scene() -> Scene {
    parse_scene(INCREMENT21_SCENE_JSON)
        .expect("increment21 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT22_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0 }
    }
  ]
}"#;

pub fn increment22_scene_json() -> &'static str {
    INCREMENT22_SCENE_JSON
}

pub fn increment22_scene() -> Scene {
    parse_scene(INCREMENT22_SCENE_JSON)
        .expect("increment22 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT23_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0, "sheen": 1.0, "sheen_roughness": 0.4, "sheen_color": [0.75, 0.12, 0.28] }
    }
  ]
}"#;

pub fn increment23_scene_json() -> &'static str {
    INCREMENT23_SCENE_JSON
}

pub fn increment23_scene() -> Scene {
    parse_scene(INCREMENT23_SCENE_JSON)
        .expect("increment23 scene JSON is valid")
        .with_default_mesh_search()
}


pub const INCREMENT24_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0, "sheen": 1.0, "sheen_roughness": 0.4, "sheen_color": [0.75, 0.12, 0.28] }
    }
  ]
}"#;

pub fn increment24_scene_json() -> &'static str {
    INCREMENT24_SCENE_JSON
}

pub fn increment24_scene() -> Scene {
    parse_scene(INCREMENT24_SCENE_JSON)
        .expect("increment24 scene JSON is valid")
        .with_default_mesh_search()
}


pub const INCREMENT25_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08, "anisotropy": 0.95, "anisotropy_rotation": 0.6 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0, "sheen": 1.0, "sheen_roughness": 0.4, "sheen_color": [0.75, 0.12, 0.28] }
    }
  ]
}"#;

pub fn increment25_scene_json() -> &'static str {
    INCREMENT25_SCENE_JSON
}

pub fn increment25_scene() -> Scene {
    parse_scene(INCREMENT25_SCENE_JSON)
        .expect("increment25 scene JSON is valid")
        .with_default_mesh_search()
}


pub const INCREMENT26_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08, "anisotropy": 0.95, "anisotropy_rotation": 0.6, "iridescence": 1.0, "iridescence_ior": 1.3, "iridescence_thickness": 380 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0, "sheen": 1.0, "sheen_roughness": 0.4, "sheen_color": [0.75, 0.12, 0.28] }
    }
  ]
}"#;

pub fn increment26_scene_json() -> &'static str {
    INCREMENT26_SCENE_JSON
}

pub fn increment26_scene() -> Scene {
    parse_scene(INCREMENT26_SCENE_JSON)
        .expect("increment26 scene JSON is valid")
        .with_default_mesh_search()
}

pub const INCREMENT27_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08, "anisotropy": 0.95, "anisotropy_rotation": 0.6, "iridescence": 1.0, "iridescence_ior": 1.3, "iridescence_thickness": 380 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0, "dispersion": 0.18 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0, "sheen": 1.0, "sheen_roughness": 0.4, "sheen_color": [0.75, 0.12, 0.28] }
    }
  ]
}"#;

pub fn increment27_scene_json() -> &'static str {
    INCREMENT27_SCENE_JSON
}

/// Increment-26 courtyard unchanged (camera / bodies / lights). Pane picks up
/// authored KHR_materials_dispersion from meshes/pane.gltf (or 0.18 if unset).
pub fn increment27_scene() -> Scene {
    let mut scene = increment26_scene();
    let disp = scene
        .bodies
        .iter()
        .find(|b| b.id == "pane")
        .and_then(|b| match &b.shape {
            Shape::Mesh { path, .. } => Some(path.clone()),
            _ => None,
        })
        .and_then(|path| scene.load_body_mesh(&path).ok())
        .and_then(|m| m.gltf_material)
        .map(|gm| gm.dispersion)
        .filter(|d| *d > 0.0)
        .unwrap_or(0.18);
    if let Some(pane) = scene.bodies.iter_mut().find(|b| b.id == "pane") {
        pane.material.dispersion = disp;
    }
    scene
}

pub const INCREMENT28_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08, "anisotropy": 0.95, "anisotropy_rotation": 0.6, "iridescence": 1.0, "iridescence_ior": 1.3, "iridescence_thickness": 380 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0, "dispersion": 0.18 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0, "sheen": 1.0, "sheen_roughness": 0.4, "sheen_color": [0.75, 0.12, 0.28] }
    },
    {
      "id": "lantern",
      "shape": { "type": "sphere", "radius": 0.12 },
      "position": [1.10, 1.22, 1.42],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.4,
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0] }
  ]
}"#;

pub fn increment28_scene_json() -> &'static str {
    INCREMENT28_SCENE_JSON
}

/// Increment-27 courtyard plus one hanging lantern on the existing pillar.
/// Clones increment27_scene() so the courtyard cannot drift, then authors
/// the lantern body and a world-space Rapier hinge.
pub fn increment28_scene() -> Scene {
    let mut scene = increment27_scene();
    scene.bodies.push(Body {
        id: "lantern".into(),
        shape: Shape::Sphere { radius: 0.12 },
        position: [1.10, 1.22, 1.42],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.4,
        linear_velocity: [0.0, 0.0, 0.0],
        material: Material {
            albedo: [0.78, 0.48, 0.16],
            roughness: 0.28,
            metallic: 0.85,
            albedo_map: None,
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            sheen: 0.0,
            sheen_roughness: 0.5,
            sheen_color: [1.0, 1.0, 1.0],
            anisotropy: 0.0,
            anisotropy_rotation: 0.0,
            iridescence: 0.0,
            iridescence_ior: 1.3,
            iridescence_thickness: 400.0,
            dispersion: 0.0,
        },
    });
    scene.joints.push(Joint::Hinge {
        body_a: "pillar".into(),
        body_b: "lantern".into(),
        // World-space attachment just above / +Z of the pillar top.
        // Converted to local anchors on each body at spawn.
        anchor: [1.10, 1.08, 1.10],
        // World X: horizontal so gravity swings the lantern down (Y-up).
        axis: [1.0, 0.0, 0.0],
    });
    scene
}

pub const INCREMENT29_SCENE_JSON: &str = r#"{
  "camera": { "position": [3.6, 2.35, 5.2], "look_at": [0.1, 0.38, 0.0], "fov_y_deg": 40 },
  "lights": [
    { "type": "directional", "direction": [-0.45, -1.0, -0.35], "color": [1.0, 0.97, 0.92], "intensity": 3.0 },
    { "type": "area", "position": [0.15, 1.45, 0.40], "size": [1.2, 0.8], "color": [1.0, 0.75, 0.45], "intensity": 40.0, "normal": [0.0, -1.0, 0.0] }
  ],
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
      "material": { "albedo": [0.92, 0.78, 0.45], "roughness": 0.15, "metallic": 0.9, "clearcoat": 1.0, "clearcoat_roughness": 0.08, "anisotropy": 0.95, "anisotropy_rotation": 0.6, "iridescence": 1.0, "iridescence_ior": 1.3, "iridescence_thickness": 380 }
    },
    {
      "id": "pillar",
      "shape": { "type": "mesh", "path": "meshes/pillar.gltf", "collider": "convex_hull" },
      "position": [1.10, 0.002, 0.70],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 16.0,
      "material": { "albedo": [0.40, 0.40, 0.42], "roughness": 0.85, "metallic": 0.0 }
    },
    {
      "id": "pane",
      "shape": { "type": "mesh", "path": "meshes/pane.gltf", "collider": "trimesh" },
      "position": [0.50, 0.08, 2.20],
      "rotation_wxyz": [0.9914449, 0.0, -0.1305262, 0.0],
      "mass": 0,
      "material": { "albedo": [0.75, 0.90, 1.00], "roughness": 0.08, "metallic": 0.0, "dispersion": 0.18 }
    },
    {
      "id": "crate",
      "shape": { "type": "mesh", "path": "meshes/crate.obj", "collider": "convex_hull" },
      "position": [-0.35, 0.002, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 2.5,
      "material": { "albedo": [0.62, 0.40, 0.22], "roughness": 0.78, "metallic": 0.0 }
    },
    {
      "id": "bench",
      "shape": { "type": "mesh", "path": "meshes/bench.obj", "collider": "trimesh" },
      "position": [1.35, 0.002, -0.15],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 5.0,
      "material": { "albedo": [0.32, 0.36, 0.40], "roughness": 0.72, "metallic": 0.0, "sheen": 1.0, "sheen_roughness": 0.4, "sheen_color": [0.75, 0.12, 0.28] }
    },
    {
      "id": "lantern",
      "shape": { "type": "sphere", "radius": 0.12 },
      "position": [1.10, 1.22, 1.42],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.4,
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 2.5],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0] },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02] }
  ]
}"#;

pub fn increment29_scene_json() -> &'static str {
    INCREMENT29_SCENE_JSON
}

/// Increment-28 courtyard plus one drawer on the crate, constrained by a
/// Rapier prismatic / slider. Clones increment28_scene() so the courtyard
/// (including lantern + hinge) cannot drift.
pub fn increment29_scene() -> Scene {
    let mut scene = increment28_scene();
    scene.bodies.push(Body {
        id: "drawer".into(),
        shape: Shape::Box { size: [0.22, 0.11, 0.16] },
        position: [-0.35, 0.10, 1.02],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.3,
        linear_velocity: [0.0, 0.0, 2.5],
        material: Material {
            albedo: [0.50, 0.32, 0.16],
            roughness: 0.72,
            metallic: 0.0,
            albedo_map: None,
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            sheen: 0.0,
            sheen_roughness: 0.5,
            sheen_color: [1.0, 1.0, 1.0],
            anisotropy: 0.0,
            anisotropy_rotation: 0.0,
            iridescence: 0.0,
            iridescence_ior: 1.3,
            iridescence_thickness: 400.0,
            dispersion: 0.0,
        },
    });
    scene.joints.push(Joint::Slider {
        body_a: "crate".into(),
        body_b: "drawer".into(),
        axis: [0.0, 0.0, 1.0],
        limits: [0.0, 0.35],
        anchor: Some([-0.35, 0.10, 1.02]),
    });
    scene
}
