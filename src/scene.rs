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
    /// Authorable sensor volumes. Sensors report overlaps and do not push bodies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<Trigger>,
    /// Authorable physics queries. After stepping, hits are recorded on the dump.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raycasts: Vec<Raycast>,
    /// Post-step hits copied from the physics dump for debug draw. Not authored.
    #[serde(skip, default)]
    pub ray_hits: Vec<RayHit>,
    /// Authorable physics shapecasts / sweeps. After stepping, hits are recorded on the dump.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shapecasts: Vec<Shapecast>,
    /// Post-step sweep hits copied from the physics dump for debug draw. Not authored.
    #[serde(skip, default)]
    pub sweep_hits: Vec<SweepHit>,
    /// Authorable one-shot impulses. Applied once at spawn (world-space, at COM).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impulses: Vec<Impulse>,
    /// When true, the physics dump records started/stopped contact events
    /// across every step. Serde default false; omitted when false so
    /// increment 18-46 JSON stay compact.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub record_contact_events: bool,
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
        /// Optional [min, max] angle limits in radians. None = unlimited.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limits: Option<[f32; 2]>,
        /// Target angular velocity (rad/s). 0 + max_force 0 = hang damper.
        #[serde(default)]
        motor_target_velocity: f32,
        /// Motor factor / max force. 0 + target 0 = hang damper (velocity 0, factor 8).
        #[serde(default)]
        motor_max_force: f32,
        /// Target angle (radians). When Some and motor_max_force > 0, Rapier
        /// position motor instead of velocity. Serde default none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        motor_target_position: Option<f32>,
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
        /// Target linear velocity along the axis (m/s). 0 + max_force 0 = no motor.
        #[serde(default)]
        motor_target_velocity: f32,
        /// Motor factor / max force. 0 + target 0 = no motor (increment 29–33 open from velocity).
        #[serde(default)]
        motor_max_force: f32,
    },
    /// Spherical / ball socket. `anchor` is world-space; converted to local
    /// on each body at spawn. Free in 2 axes (not locked to a swing plane).
    #[serde(rename = "ball")]
    Ball {
        body_a: String,
        body_b: String,
        anchor: [f32; 3],
    },
    /// Weld / fixed joint. `anchor` is world-space; converted to local on
    /// each body at spawn. Locks all relative degrees of freedom.
    #[serde(rename = "fixed")]
    Fixed {
        body_a: String,
        body_b: String,
        anchor: [f32; 3],
    },
    /// Rope / distance joint. `anchor` is world-space; converted to local
    /// on each body at spawn. Max length is `rest_length` (Rapier RopeJoint).
    #[serde(rename = "distance")]
    Distance {
        body_a: String,
        body_b: String,
        anchor: [f32; 3],
        rest_length: f32,
        /// Impulse magnitude that snaps the rope. 0 = never break.
        #[serde(default, skip_serializing_if = "is_zero_f32")]
        break_force: f32,
    },
    /// Spring-damper. `anchor` is world-space; converted to local on
    /// each body at spawn. Rest length / stiffness / damping map to
    /// Rapier SpringJoint (ForceBased motor on coupled LIN_AXES).
    #[serde(rename = "spring")]
    Spring {
        body_a: String,
        body_b: String,
        anchor: [f32; 3],
        rest_length: f32,
        stiffness: f32,
        damping: f32,
    },
}

/// Authorable sensor volume. Does not generate contact forces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub shape: Shape,
    pub position: [f32; 3],
}

/// Authorable physics raycast. `direction` should be unit-length; `max_toi` is
/// the maximum travel in world units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Raycast {
    pub id: String,
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub max_toi: f32,
}

/// Post-step raycast hit. Misses are omitted from the dump.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RayHit {
    pub ray: String,
    pub body: String,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub toi: f32,
}

/// Authorable physics shapecast / sweep. `direction` should be unit-length;
/// `max_toi` is the maximum travel in world units. `shape` is the swept volume
/// (`type: "box"` + full `size` xyz).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shapecast {
    pub id: String,
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub shape: Shape,
    pub max_toi: f32,
}

/// Post-step shapecast hit. Misses are omitted from the dump.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepHit {
    pub sweep: String,
    pub body: String,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub toi: f32,
}

/// Authorable linear impulse. Applied once at spawn in world space at COM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Impulse {
    pub body: String,
    pub linear: [f32; 3],
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

/// Authored kinematic character controller. World-space wish velocity (m/s).
/// Horizontal wish is authored; the engine adds a downward component so
/// the walker stays on the floor via Rapier snap_to_ground / gravity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterController {
    pub desired_velocity: [f32; 3],
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
    /// When true, spawn as Rapier kinematic_velocity_based and re-apply
    /// authored `linear_velocity` every physics step (constant authored vel).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub kinematic: bool,
    /// Optional Rapier kinematic character controller. When Some, spawn
    /// as kinematic_position_based and drive with move_shape each step.
    /// Serde default none; omitted when none so increment 18–47 JSON stay compact.
    /// Do not combine with `kinematic: true` (that is the increment-39 linvel drive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller: Option<CharacterController>,
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
                    emissive: [0.0, 0.0, 0.0],
                    emissive_intensity: 0.0,
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
    /// Self-glow / mesh-light color. Optional; serde default [0,0,0].
    #[serde(default, skip_serializing_if = "is_zero_emissive")]
    pub emissive: [f32; 3],
    /// Mesh-light intensity. 0 = off (increment-16 texture emissive only, no mesh light).
    #[serde(default, skip_serializing_if = "is_zero_f32")]
    pub emissive_intensity: f32,
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

fn is_zero_emissive(v: &[f32; 3]) -> bool {
    v[0] == 0.0 && v[1] == 0.0 && v[2] == 0.0
}

fn is_zero_f32(v: &f32) -> bool {
    *v == 0.0
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
        kinematic: false,
        controller: None,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
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
        limits: None,
        motor_target_velocity: 0.0,
        motor_max_force: 0.0,
        motor_target_position: None,
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
        kinematic: false,
        controller: None,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene.joints.push(Joint::Slider {
        body_a: "crate".into(),
        body_b: "drawer".into(),
        axis: [0.0, 0.0, 1.0],
        limits: [0.0, 0.35],
        anchor: Some([-0.35, 0.10, 1.02]),
        motor_target_velocity: 0.0,
        motor_max_force: 0.0,
    });
    scene
}

pub const INCREMENT30_SCENE_JSON: &str = r#"{
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
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02] }
  ]
}"#;

pub fn increment30_scene_json() -> &'static str {
    INCREMENT30_SCENE_JSON
}

/// Increment-29 courtyard plus an authored motor on the existing pillar–lantern
/// hinge. Clones increment29_scene() so the courtyard (drawer + slider included)
/// cannot drift. No new bodies, no new joints.
pub fn increment30_scene() -> Scene {
    let mut scene = increment29_scene();
    for joint in &mut scene.joints {
        if let Joint::Hinge {
            body_a,
            body_b,
            motor_target_velocity,
            motor_max_force,
            ..
        } = joint
        {
            if body_a == "pillar" && body_b == "lantern" {
                *motor_target_velocity = 4.0;
                *motor_max_force = 8.0;
            }
        }
    }
    scene
}

pub const INCREMENT31_SCENE_JSON: &str = r#"{
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
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02] },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] }
  ]
}"#;

pub fn increment31_scene_json() -> &'static str {
    INCREMENT31_SCENE_JSON
}

/// Increment-30 courtyard plus one charm hanging from the lantern on a
/// Rapier spherical / ball socket. Clones increment30_scene() so the
/// courtyard (hinge motor + drawer slider included) cannot drift.
pub fn increment31_scene() -> Scene {
    let mut scene = increment30_scene();
    scene.bodies.push(Body {
        id: "charm".into(),
        shape: Shape::Sphere { radius: 0.06 },
        // Offset +X (off the lantern hinge swing plane YZ) and slightly +Y/+Z
        // so the charm is not already hanging at spawn.
        position: [1.32, 1.30, 1.48],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.15,
        linear_velocity: [0.0, 0.0, 0.0],
        kinematic: false,
        controller: None,
        material: Material {
            albedo: [0.88, 0.70, 0.22],
            roughness: 0.22,
            metallic: 0.92,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene.joints.push(Joint::Ball {
        body_a: "lantern".into(),
        body_b: "charm".into(),
        // World-space on/near the lantern, slightly below the lantern COM
        // (radius 0.12). Converted to local anchors on each body at spawn.
        anchor: [1.10, 1.16, 1.42],
    });
    scene
}

pub const INCREMENT32_SCENE_JSON: &str = r#"{
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
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02] },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ]
}"#;

pub fn increment32_scene_json() -> &'static str {
    INCREMENT32_SCENE_JSON
}

/// Increment-31 courtyard plus one drawer-open sensor volume.
/// Clones increment31_scene() so the courtyard (charm + ball, hinge motor,
/// drawer slider) cannot drift. No new bodies, no new joints.
pub fn increment32_scene() -> Scene {
    let mut scene = increment31_scene();
    scene.triggers.push(Trigger {
        id: "drawer_open".into(),
        shape: Shape::Box {
            size: [0.30, 0.22, 0.28],
        },
        position: [-0.35, 0.10, 1.37],
    });
    scene
}

pub const INCREMENT33_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 2.5],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02] },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ]
}"#;

pub fn increment33_scene_json() -> &'static str {
    INCREMENT33_SCENE_JSON
}

/// Increment-32 courtyard plus lantern self-glow / mesh light.
/// Clones increment32_scene() so the courtyard (drawer_open trigger included)
/// cannot drift. Sets ONLY the existing lantern material emissive fields.
/// No new bodies, no new joints, no new lights[] entries.
pub fn increment33_scene() -> Scene {
    let mut scene = increment32_scene();
    if let Some(lantern) = scene.bodies.iter_mut().find(|b| b.id == "lantern") {
        lantern.material.emissive = [1.0, 0.55, 0.12];
        lantern.material.emissive_intensity = 16.0;
    }
    scene
}

pub const INCREMENT34_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ]
}"#;

pub fn increment34_scene_json() -> &'static str {
    INCREMENT34_SCENE_JSON
}

/// Increment-33 courtyard plus an authored motor on the existing crate–drawer
/// slider. Clones increment33_scene() so the courtyard (emissive lantern,
/// drawer_open trigger, charm + ball, hinge motor) cannot drift. Sets ONLY
/// the crate–drawer slider motor and zeros the increment-34 drawer initial
/// +Z velocity so the motor cleanly drives closed. No new bodies, no new
/// joints, no new lights[] entries.
pub fn increment34_scene() -> Scene {
    let mut scene = increment33_scene();
    if let Some(drawer) = scene.bodies.iter_mut().find(|b| b.id == "drawer") {
        drawer.linear_velocity = [0.0, 0.0, 0.0];
    }
    for joint in &mut scene.joints {
        if let Joint::Slider {
            body_a,
            body_b,
            motor_target_velocity,
            motor_max_force,
            ..
        } = joint
        {
            if body_a == "crate" && body_b == "drawer" {
                *motor_target_velocity = -2.0;
                *motor_max_force = 6.0;
            }
        }
    }
    scene
}

pub const INCREMENT35_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ]
}"#;

pub fn increment35_scene_json() -> &'static str {
    INCREMENT35_SCENE_JSON
}

/// Increment-34 courtyard plus one authored ray that probes the closed drawer.
/// Clones increment34_scene() so the courtyard (slider motor -2/6, emissive
/// lantern, drawer_open trigger, charm + ball, hinge motor) cannot drift.
/// Adds ONLY `drawer_probe`. Camera stays increment-34. No new bodies, joints,
/// or lights. increment34_scene() stays raycast-free.
pub fn increment35_scene() -> Scene {
    let mut scene = increment34_scene();
    let origin: [f32; 3] = [-0.35, 0.55, 1.35];
    let target: [f32; 3] = [-0.35, 0.10, 1.02];
    let d = [
        target[0] - origin[0],
        target[1] - origin[1],
        target[2] - origin[2],
    ];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    scene.raycasts.push(Raycast {
        id: "drawer_probe".into(),
        origin,
        direction: [d[0] / len, d[1] / len, d[2] / len],
        max_toi: 2.0,
    });
    scene
}

pub const INCREMENT36_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ]
}"#;

pub fn increment36_scene_json() -> &'static str {
    INCREMENT36_SCENE_JSON
}

/// Increment-35 courtyard plus one authored shapecast that sweeps the closed
/// drawer. Clones increment35_scene() so the courtyard (drawer_probe ray,
/// slider motor -2/6, emissive lantern, drawer_open trigger, charm + ball,
/// hinge motor) cannot drift. Adds ONLY `drawer_sweep`. Camera stays
/// increment-35. No new bodies, joints, or lights. increment35_scene() stays
/// shapecast-free.
pub fn increment36_scene() -> Scene {
    let mut scene = increment35_scene();
    scene.shapecasts.push(Shapecast {
        id: "drawer_sweep".into(),
        origin: [-0.35, 0.55, 1.02],
        direction: [0.0, -1.0, 0.0],
        shape: Shape::Box {
            size: [0.10, 0.04, 0.10],
        },
        max_toi: 1.0,
    });
    scene
}

pub const INCREMENT37_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ]
}"#;

pub fn increment37_scene_json() -> &'static str {
    INCREMENT37_SCENE_JSON
}

/// Increment-36 courtyard plus a crate lid welded with a Rapier fixed joint.
/// Clones increment36_scene() so the courtyard (drawer_sweep, drawer_probe,
/// slider motor -2/6, emissive lantern, drawer_open trigger, charm + ball,
/// hinge motor) cannot drift. Adds ONLY the `lid` body and one `Fixed` joint.
/// Camera stays increment-36. No new lights. increment36_scene() stays
/// lid-free and fixed-joint-free.
pub fn increment37_scene() -> Scene {
    let mut scene = increment36_scene();
    scene.bodies.push(Body {
        id: "lid".into(),
        shape: Shape::Box { size: [0.28, 0.04, 0.28] },
        position: [-0.35, 0.28, 0.85],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.2,
        linear_velocity: [0.0, 0.0, 0.0],
        kinematic: false,
        controller: None,
        material: Material {
            albedo: [0.45, 0.28, 0.14],
            roughness: 0.75,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene.joints.push(Joint::Fixed {
        body_a: "crate".into(),
        body_b: "lid".into(),
        // World-space at the crate–lid interface (crate top / lid bottom).
        // Converted to local anchors on each body at spawn.
        anchor: [-0.35, 0.26, 0.85],
    });
    scene
}

pub const INCREMENT38_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] }
  ]
}
"#;

pub fn increment38_scene_json() -> &'static str {
    INCREMENT38_SCENE_JSON
}

/// Increment-37 courtyard plus one authored impulse on the gold ball.
/// Clones increment37_scene() so the courtyard (lid + fixed, drawer_sweep,
/// drawer_probe, slider motor -2/6, emissive lantern, drawer_open trigger,
/// charm + ball, hinge motor) cannot drift. Adds ONLY
/// `{ body: "ball", linear: [1.8, 0.4, 0.5] }`. Camera stays increment-37.
/// No new bodies, joints, or lights. increment37_scene() stays impulse-free.
pub fn increment38_scene() -> Scene {
    let mut scene = increment37_scene();
    scene.impulses.push(Impulse {
        body: "ball".into(),
        linear: [1.8, 0.4, 0.5],
    });
    scene
}


pub const INCREMENT39_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] }
  ]
}
"#;

pub fn increment39_scene_json() -> &'static str {
    INCREMENT39_SCENE_JSON
}

/// Increment-38 courtyard plus one kinematic moving platform.
/// Clones increment38_scene() so the courtyard (lid + fixed, ball impulse,
/// drawer_sweep, drawer_probe, slider motor -2/6, emissive lantern,
/// drawer_open trigger, charm + ball, hinge motor) cannot drift. Adds ONLY
/// the `platform` body (`kinematic: true`, slides +X). Camera stays
/// increment-38. No new lights. increment38_scene() stays platform-free
/// and kinematic-free.
pub fn increment39_scene() -> Scene {
    let mut scene = increment38_scene();
    scene.bodies.push(Body {
        id: "platform".into(),
        shape: Shape::Box { size: [0.55, 0.06, 0.35] },
        position: [-0.55, 0.04, -0.55],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.0,
        linear_velocity: [0.45, 0.0, 0.0],
        kinematic: true,
        controller: None,
        material: Material {
            albedo: [0.38, 0.40, 0.44],
            roughness: 0.55,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene
}

pub const INCREMENT40_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    },
    {
      "id": "rider",
      "shape": { "type": "box", "size": [0.16, 0.16, 0.16] },
      "position": [-0.55, 0.15, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.35,
      "material": { "albedo": [0.72, 0.38, 0.22], "roughness": 0.7, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] }
  ]
}
"#;

pub fn increment40_scene_json() -> &'static str {
    INCREMENT40_SCENE_JSON
}

/// Increment-39 courtyard plus one dynamic clay rider on the platform.
/// Clones increment39_scene() so the courtyard (kinematic platform, lid +
/// fixed, ball impulse, drawer_sweep, drawer_probe, slider motor -2/6,
/// emissive lantern, drawer_open trigger, charm + ball, hinge motor) cannot
/// drift. Adds ONLY the `rider` body (dynamic clay cube seated on the
/// platform). Camera stays increment-39. No new lights.
/// increment39_scene() stays rider-free.
pub fn increment40_scene() -> Scene {
    let mut scene = increment39_scene();
    scene.bodies.push(Body {
        id: "rider".into(),
        shape: Shape::Box { size: [0.16, 0.16, 0.16] },
        position: [-0.55, 0.15, -0.55],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.35,
        linear_velocity: [0.0, 0.0, 0.0],
        kinematic: false,
        controller: None,
        material: Material {
            albedo: [0.72, 0.38, 0.22],
            roughness: 0.7,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene
}

pub const INCREMENT41_SCENE_JSON: &str = r#"{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    },
    {
      "id": "rider",
      "shape": { "type": "box", "size": [0.16, 0.16, 0.16] },
      "position": [-0.55, 0.15, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.35,
      "material": { "albedo": [0.72, 0.38, 0.22], "roughness": 0.7, "metallic": 0.0 }
    },
    {
      "id": "gate",
      "shape": { "type": "box", "size": [0.06, 0.72, 0.42] },
      "position": [0.35, 0.40, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.5,
      "material": { "albedo": [0.18, 0.42, 0.38], "roughness": 0.45, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] },
    { "type": "hinge", "body_a": "ground", "body_b": "gate", "anchor": [0.35, 0.04, 1.75], "axis": [0.0, 1.0, 0.0], "limits": [0.0, 1.15], "motor_target_velocity": 1.4, "motor_max_force": 5.0 }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] }
  ]
}
"#;

pub fn increment41_scene_json() -> &'static str {
    INCREMENT41_SCENE_JSON
}

/// Increment-40 courtyard plus one motor-driven teal gate on a limited hinge.
/// Clones increment40_scene() so the courtyard (rider, kinematic platform,
/// lid + fixed, ball impulse, drawer_sweep, drawer_probe, slider motor -2/6,
/// emissive lantern, drawer_open trigger, charm + ball, hinge motor) cannot
/// drift. Adds ONLY the `gate` body (dynamic teal box, camera-facing
/// foreground) and a `ground`–`gate` hinge with authored limits + motor.
/// Camera stays increment-40. No new lights. increment40_scene() stays
/// gate-free; the pillar–lantern hinge stays limit-free.
pub fn increment41_scene() -> Scene {
    let mut scene = increment40_scene();
    scene.bodies.push(Body {
        id: "gate".into(),
        shape: Shape::Box { size: [0.06, 0.72, 0.42] },
        position: [0.35, 0.40, 1.75],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.5,
        linear_velocity: [0.0, 0.0, 0.0],
        kinematic: false,
        controller: None,
        material: Material {
            albedo: [0.18, 0.42, 0.38],
            roughness: 0.45,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene.joints.push(Joint::Hinge {
        body_a: "ground".into(),
        body_b: "gate".into(),
        // World-space attachment at the gate foot. Converted to local
        // anchors on each body at spawn (same helper as increment 28).
        anchor: [0.35, 0.04, 1.75],
        // World Y: yaw so the camera-facing gate swings open.
        axis: [0.0, 1.0, 0.0],
        limits: Some([0.0, 1.15]),
        motor_target_velocity: 1.4,
        motor_max_force: 5.0,
        motor_target_position: None,
    });
    scene
}

pub const INCREMENT42_SCENE_JSON: &str = r#"
{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    },
    {
      "id": "rider",
      "shape": { "type": "box", "size": [0.16, 0.16, 0.16] },
      "position": [-0.55, 0.15, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.35,
      "material": { "albedo": [0.72, 0.38, 0.22], "roughness": 0.7, "metallic": 0.0 }
    },
    {
      "id": "gate",
      "shape": { "type": "box", "size": [0.06, 0.72, 0.42] },
      "position": [0.35, 0.40, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.5,
      "material": { "albedo": [0.18, 0.42, 0.38], "roughness": 0.45, "metallic": 0.0 }
    },
    {
      "id": "bob",
      "shape": { "type": "sphere", "radius": 0.08 },
      "position": [0.35, 0.88, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.82, 0.64, 0.22], "roughness": 0.28, "metallic": 0.85 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] },
    { "type": "hinge", "body_a": "ground", "body_b": "gate", "anchor": [0.35, 0.04, 1.75], "axis": [0.0, 1.0, 0.0], "limits": [0.0, 1.15], "motor_target_velocity": 1.4, "motor_max_force": 5.0 },
    { "type": "distance", "body_a": "gate", "body_b": "bob", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.38 }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] }
  ]
}
"#;

pub fn increment42_scene_json() -> &'static str {
    INCREMENT42_SCENE_JSON
}

/// Increment-41 courtyard plus one brass bob hung from the gate on a rope.
/// Clones increment41_scene() so the courtyard (gate + limited hinge, rider,
/// kinematic platform, lid + fixed, ball impulse, drawer_sweep, drawer_probe,
/// slider motor -2/6, emissive lantern, drawer_open trigger, charm + ball,
/// hinge motor) cannot drift. Adds ONLY the `bob` body (dynamic brass sphere
/// above the gate top) and a `gate`–`bob` Distance joint (Rapier RopeJoint,
/// max length = rest_length). Camera stays increment-41. No new lights.
/// increment41_scene() stays bob-free and distance-joint-free.
pub fn increment42_scene() -> Scene {
    let mut scene = increment41_scene();
    scene.bodies.push(Body {
        id: "bob".into(),
        shape: Shape::Sphere { radius: 0.08 },
        position: [0.35, 0.88, 1.75],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.2,
        linear_velocity: [0.0, 0.0, 0.0],
        kinematic: false,
        controller: None,
        material: Material {
            albedo: [0.82, 0.64, 0.22],
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene.joints.push(Joint::Distance {
        body_a: "gate".into(),
        body_b: "bob".into(),
        // World-space attachment at the gate top. Converted to local
        // anchors on each body at spawn (same helper as increment 28).
        // Local anchors track the yawing gate so the bob stays hung
        // from the moving top.
        anchor: [0.35, 0.76, 1.75],
        rest_length: 0.38,
        break_force: 0.0,
    });
    scene
}

pub const INCREMENT43_SCENE_JSON: &str = r#"
{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    },
    {
      "id": "rider",
      "shape": { "type": "box", "size": [0.16, 0.16, 0.16] },
      "position": [-0.55, 0.15, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.35,
      "material": { "albedo": [0.72, 0.38, 0.22], "roughness": 0.7, "metallic": 0.0 }
    },
    {
      "id": "gate",
      "shape": { "type": "box", "size": [0.06, 0.72, 0.42] },
      "position": [0.35, 0.40, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.5,
      "material": { "albedo": [0.18, 0.42, 0.38], "roughness": 0.45, "metallic": 0.0 }
    },
    {
      "id": "bob",
      "shape": { "type": "sphere", "radius": 0.08 },
      "position": [0.35, 0.88, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.82, 0.64, 0.22], "roughness": 0.28, "metallic": 0.85 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] },
    { "type": "hinge", "body_a": "ground", "body_b": "gate", "anchor": [0.35, 0.04, 1.75], "axis": [0.0, 1.0, 0.0], "limits": [0.0, 1.15], "motor_target_velocity": 1.4, "motor_max_force": 5.0 },
    { "type": "distance", "body_a": "gate", "body_b": "bob", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.38, "break_force": 1.5 }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] },
    { "body": "bob", "linear": [0.0, -4.0, 1.6] }
  ]
}
"#;

pub fn increment43_scene_json() -> &'static str {
    INCREMENT43_SCENE_JSON
}

/// Increment-42 courtyard plus a breakable gate–bob rope.
/// Clones increment42_scene() so the courtyard (gate + limited hinge, rider,
/// kinematic platform, lid + fixed, ball impulse, drawer_sweep, drawer_probe,
/// slider motor -2/6, emissive lantern, drawer_open trigger, charm + ball,
/// hinge motor, brass bob + distance) cannot drift. ONLY sets `break_force`
/// ~1.5 on the existing `gate`–`bob` Distance joint and appends one extra
/// impulse on the bob (`linear` `[0.0, -4.0, 1.6]`) so the rope snaps.
/// Camera stays increment-42. No new lights. No new body.
/// increment42_scene() stays unbreakable (`break_force` 0) and does not
/// add the bob impulse.
pub fn increment43_scene() -> Scene {
    let mut scene = increment42_scene();
    for joint in &mut scene.joints {
        if let Joint::Distance {
            body_a,
            body_b,
            break_force,
            ..
        } = joint
        {
            if body_a == "gate" && body_b == "bob" {
                *break_force = 1.5;
            }
        }
    }
    scene.impulses.push(Impulse {
        body: "bob".into(),
        linear: [0.0, -4.0, 1.6],
    });
    scene
}

pub const INCREMENT44_SCENE_JSON: &str = r#"
{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    },
    {
      "id": "rider",
      "shape": { "type": "box", "size": [0.16, 0.16, 0.16] },
      "position": [-0.55, 0.15, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.35,
      "material": { "albedo": [0.72, 0.38, 0.22], "roughness": 0.7, "metallic": 0.0 }
    },
    {
      "id": "gate",
      "shape": { "type": "box", "size": [0.06, 0.72, 0.42] },
      "position": [0.35, 0.40, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.5,
      "material": { "albedo": [0.18, 0.42, 0.38], "roughness": 0.45, "metallic": 0.0 }
    },
    {
      "id": "bob",
      "shape": { "type": "sphere", "radius": 0.08 },
      "position": [0.35, 0.88, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.82, 0.64, 0.22], "roughness": 0.28, "metallic": 0.85 }
    },
    {
      "id": "cork",
      "shape": { "type": "sphere", "radius": 0.14 },
      "position": [0.35, 1.15, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.25,
      "material": { "albedo": [0.72, 0.58, 0.32], "roughness": 0.8, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] },
    { "type": "hinge", "body_a": "ground", "body_b": "gate", "anchor": [0.35, 0.04, 1.75], "axis": [0.0, 1.0, 0.0], "limits": [0.0, 1.15], "motor_target_velocity": 1.4, "motor_max_force": 5.0 },
    { "type": "distance", "body_a": "gate", "body_b": "bob", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.38, "break_force": 1.5 },
    { "type": "spring", "body_a": "gate", "body_b": "cork", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.42, "stiffness": 40, "damping": 4 }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] },
    { "body": "bob", "linear": [0.0, -4.0, 1.6] }
  ]
}
"#;

pub fn increment44_scene_json() -> &'static str {
    INCREMENT44_SCENE_JSON
}

/// Increment-43 courtyard plus a cork hung from the gate on a spring.
/// Clones increment43_scene() so the courtyard (broken rope + fallen bob,
/// gate + hinge limits, rider, platform, lid + fixed, impulses,
/// drawer_probe, drawer_sweep) cannot drift. Adds ONLY the `cork` body
/// (dynamic tan sphere above the gate top) and a `gate`–`cork` Spring
/// joint (Rapier SpringJoint). Camera stays increment-43. No new lights.
/// increment43_scene() stays cork-free and spring-free.
pub fn increment44_scene() -> Scene {
    let mut scene = increment43_scene();
    scene.bodies.push(Body {
        id: "cork".into(),
        shape: Shape::Sphere { radius: 0.14 },
        position: [0.35, 1.15, 1.75],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.25,
        linear_velocity: [0.0, 0.0, 0.0],
        kinematic: false,
        controller: None,
        material: Material {
            albedo: [0.72, 0.58, 0.32],
            roughness: 0.8,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene.joints.push(Joint::Spring {
        body_a: "gate".into(),
        body_b: "cork".into(),
        // World-space attachment at the gate top. Converted to local
        // anchors on each body at spawn (same helper as increment 28).
        // Cork attaches at COM so rest_length is COM-to-anchor.
        anchor: [0.35, 0.76, 1.75],
        rest_length: 0.42,
        stiffness: 40.0,
        damping: 4.0,
    });
    scene
}

pub const INCREMENT45_SCENE_JSON: &str = r#"
{
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    },
    {
      "id": "rider",
      "shape": { "type": "box", "size": [0.16, 0.16, 0.16] },
      "position": [-0.55, 0.15, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.35,
      "material": { "albedo": [0.72, 0.38, 0.22], "roughness": 0.7, "metallic": 0.0 }
    },
    {
      "id": "gate",
      "shape": { "type": "box", "size": [0.06, 0.72, 0.42] },
      "position": [0.35, 0.40, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.5,
      "material": { "albedo": [0.18, 0.42, 0.38], "roughness": 0.45, "metallic": 0.0 }
    },
    {
      "id": "bob",
      "shape": { "type": "sphere", "radius": 0.08 },
      "position": [0.35, 0.88, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.82, 0.64, 0.22], "roughness": 0.28, "metallic": 0.85 }
    },
    {
      "id": "cork",
      "shape": { "type": "sphere", "radius": 0.14 },
      "position": [0.35, 1.15, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.25,
      "material": { "albedo": [0.72, 0.58, 0.32], "roughness": 0.8, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] },
    { "type": "hinge", "body_a": "ground", "body_b": "gate", "anchor": [0.35, 0.04, 1.75], "axis": [0.0, 1.0, 0.0], "limits": [0.0, 1.15], "motor_target_position": 0.55, "motor_max_force": 5.0 },
    { "type": "distance", "body_a": "gate", "body_b": "bob", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.38, "break_force": 1.5 },
    { "type": "spring", "body_a": "gate", "body_b": "cork", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.42, "stiffness": 40, "damping": 4 }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] },
    { "body": "bob", "linear": [0.0, -4.0, 1.6] }
  ]
}
"#;

pub fn increment45_scene_json() -> &'static str {
    INCREMENT45_SCENE_JSON
}

/// Increment-44 courtyard with the ground–gate hinge driven by a
/// position motor to ~0.55 rad instead of slamming the 1.15 limit.
/// Clones increment44_scene() so the courtyard (cork spring, broken
/// rope + fallen bob, rider, platform, lid + fixed, impulses,
/// drawer_probe, drawer_sweep) cannot drift. ONLY updates the
/// ground–gate hinge: `motor_target_position` ≈ 0.55, velocity 0,
/// keep limits [0, 1.15] and motor_max_force 5.0.
/// Camera stays increment-44. No new lights. No new bodies.
/// increment44_scene() stays velocity-driven (motor_target_velocity
/// 1.4, no motor_target_position).
pub fn increment45_scene() -> Scene {
    let mut scene = increment44_scene();
    for joint in &mut scene.joints {
        if let Joint::Hinge {
            body_a,
            body_b,
            motor_target_velocity,
            motor_target_position,
            ..
        } = joint
        {
            if body_a == "ground" && body_b == "gate" {
                *motor_target_position = Some(0.55);
                *motor_target_velocity = 0.0;
            }
        }
    }
    scene
}

pub const INCREMENT46_SCENE_JSON: &str = r#"
{
  "camera": { "position": [1.85, 1.35, 3.15], "look_at": [0.35, 0.42, 1.55], "fov_y_deg": 40 },
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
      "material": { "albedo": [0.78, 0.48, 0.16], "roughness": 0.28, "metallic": 0.85, "emissive": [1.0, 0.55, 0.12], "emissive_intensity": 16 }
    },
    {
      "id": "drawer",
      "shape": { "type": "box", "size": [0.22, 0.11, 0.16] },
      "position": [-0.35, 0.10, 1.02],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.3,
      "linear_velocity": [0.0, 0.0, 0.0],
      "material": { "albedo": [0.50, 0.32, 0.16], "roughness": 0.72, "metallic": 0.0 }
    },
    {
      "id": "charm",
      "shape": { "type": "sphere", "radius": 0.06 },
      "position": [1.32, 1.30, 1.48],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.15,
      "material": { "albedo": [0.88, 0.70, 0.22], "roughness": 0.22, "metallic": 0.92 }
    },
    {
      "id": "lid",
      "shape": { "type": "box", "size": [0.28, 0.04, 0.28] },
      "position": [-0.35, 0.28, 0.85],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.45, 0.28, 0.14], "roughness": 0.75, "metallic": 0.0 }
    },
    {
      "id": "platform",
      "shape": { "type": "box", "size": [0.55, 0.06, 0.35] },
      "position": [-0.55, 0.04, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0,
      "kinematic": true,
      "linear_velocity": [0.45, 0.0, 0.0],
      "material": { "albedo": [0.38, 0.40, 0.44], "roughness": 0.55, "metallic": 0.0 }
    },
    {
      "id": "rider",
      "shape": { "type": "box", "size": [0.16, 0.16, 0.16] },
      "position": [-0.55, 0.15, -0.55],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.35,
      "material": { "albedo": [0.72, 0.38, 0.22], "roughness": 0.7, "metallic": 0.0 }
    },
    {
      "id": "gate",
      "shape": { "type": "box", "size": [0.06, 0.72, 0.42] },
      "position": [0.35, 0.40, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.5,
      "material": { "albedo": [0.18, 0.42, 0.38], "roughness": 0.45, "metallic": 0.0 }
    },
    {
      "id": "bob",
      "shape": { "type": "sphere", "radius": 0.08 },
      "position": [0.35, 0.88, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.2,
      "material": { "albedo": [0.82, 0.64, 0.22], "roughness": 0.28, "metallic": 0.85 }
    },
    {
      "id": "cork",
      "shape": { "type": "sphere", "radius": 0.14 },
      "position": [0.35, 1.15, 1.75],
      "rotation_wxyz": [1, 0, 0, 0],
      "mass": 0.25,
      "material": { "albedo": [0.72, 0.58, 0.32], "roughness": 0.8, "metallic": 0.0 }
    }
  ],
  "joints": [
    { "type": "hinge", "body_a": "pillar", "body_b": "lantern", "anchor": [1.10, 1.08, 1.10], "axis": [1.0, 0.0, 0.0], "motor_target_velocity": 4.0, "motor_max_force": 8.0 },
    { "type": "slider", "body_a": "crate", "body_b": "drawer", "axis": [0.0, 0.0, 1.0], "limits": [0.0, 0.35], "anchor": [-0.35, 0.10, 1.02], "motor_target_velocity": -2.0, "motor_max_force": 6.0 },
    { "type": "ball", "body_a": "lantern", "body_b": "charm", "anchor": [1.10, 1.16, 1.42] },
    { "type": "fixed", "body_a": "crate", "body_b": "lid", "anchor": [-0.35, 0.26, 0.85] },
    { "type": "hinge", "body_a": "ground", "body_b": "gate", "anchor": [0.35, 0.04, 1.75], "axis": [0.0, 1.0, 0.0], "limits": [0.0, 1.15], "motor_target_position": 0.55, "motor_max_force": 5.0 },
    { "type": "distance", "body_a": "gate", "body_b": "bob", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.38, "break_force": 1.5 },
    { "type": "spring", "body_a": "gate", "body_b": "cork", "anchor": [0.35, 0.76, 1.75], "rest_length": 0.42, "stiffness": 40, "damping": 4 }
  ],
  "triggers": [
    { "id": "drawer_open", "shape": { "type": "box", "size": [0.30, 0.22, 0.28] }, "position": [-0.35, 0.10, 1.37] }
  ],
  "raycasts": [
    { "id": "drawer_probe", "origin": [-0.35, 0.55, 1.35], "direction": [0.0, -0.806405, -0.591364], "max_toi": 2.0 }
  ],
  "shapecasts": [
    { "id": "drawer_sweep", "origin": [-0.35, 0.55, 1.02], "direction": [0.0, -1.0, 0.0], "shape": { "type": "box", "size": [0.10, 0.04, 0.10] }, "max_toi": 1.0 }
  ],
  "impulses": [
    { "body": "ball", "linear": [1.8, 0.4, 0.5] },
    { "body": "bob", "linear": [0.0, -4.0, 1.6] }
  ]
}

"#;

pub fn increment46_scene_json() -> &'static str {
    INCREMENT46_SCENE_JSON
}

/// Increment-45 courtyard with the single camera re-aimed at the
/// gate / cork / fallen-bob cluster. Clones increment45_scene() so
/// physics (position motor 0.55, cork spring, broken rope + fallen
/// bob, rider, platform, lid + fixed, impulses, drawer_probe,
/// drawer_sweep) cannot drift. ONLY writes camera.position,
/// camera.look_at, and camera.fov_y_deg. Still one `camera`, one
/// frame. No `cameras[]`. No new bodies, lights, joints, or impulses.
/// increment45_scene() keeps the wide courtyard camera
/// [3.6, 2.35, 5.2] look_at [0.1, 0.38, 0].
pub fn increment46_scene() -> Scene {
    let mut scene = increment45_scene();
    scene.camera.position = [1.85, 1.35, 3.15];
    scene.camera.look_at = [0.35, 0.42, 1.55];
    scene.camera.fov_y_deg = 40.0;
    scene
}

pub const INCREMENT47_SCENE_JSON: &str = INCREMENT46_SCENE_JSON;

pub fn increment47_scene_json() -> &'static str {
    INCREMENT47_SCENE_JSON
}

/// Increment-46 courtyard with contact-event recording enabled.
/// Clones increment46_scene() so camera / bodies / joints / impulses
/// cannot drift. ONLY sets `record_contact_events`. No visual change.
/// increment46_scene() stays event-free.
pub fn increment47_scene() -> Scene {
    let mut scene = increment46_scene();
    scene.record_contact_events = true;
    scene
}

pub fn increment48_scene_json() -> &'static str {
    static JSON: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    JSON.get_or_init(|| {
        let mut v: serde_json::Value = serde_json::from_str(INCREMENT47_SCENE_JSON)
            .expect("increment47 scene JSON is valid");
        v["bodies"]
            .as_array_mut()
            .expect("increment47 bodies")
            .push(serde_json::json!({
                "id": "walker",
                "shape": { "type": "box", "size": [0.18, 0.36, 0.18] },
                "position": [1.15, 0.20, 1.45],
                "rotation_wxyz": [1, 0, 0, 0],
                "mass": 0,
                "controller": { "desired_velocity": [-0.55, 0.0, 0.0] },
                "material": { "albedo": [0.85, 0.22, 0.48], "roughness": 0.5, "metallic": 0.0 }
            }));
        serde_json::to_string_pretty(&v).expect("serialize increment48 scene JSON")
    })
    .as_str()
}

/// Increment-47 courtyard plus one coral walker driven by a Rapier
/// KinematicCharacterController. Clones increment47_scene() so camera /
/// record_contact_events / courtyard (gate, cork, bob, rider, platform,
/// lid, impulses, probes) cannot drift. Adds ONLY the `walker` body
/// (box, mass 0, no increment-39 `kinematic` flag) with
/// `controller.desired_velocity` ≈ [-0.55, 0, 0]. Camera stays
/// [1.85, 1.35, 3.15] look_at [0.35, 0.42, 1.55] fov 40.
/// increment47_scene() stays walker-free and controller-free.
pub fn increment48_scene() -> Scene {
    let mut scene = increment47_scene();
    scene.bodies.push(Body {
        id: "walker".into(),
        shape: Shape::Box {
            size: [0.18, 0.36, 0.18],
        },
        position: [1.15, 0.20, 1.45],
        rotation_wxyz: [1.0, 0.0, 0.0, 0.0],
        mass: 0.0,
        linear_velocity: [0.0, 0.0, 0.0],
        kinematic: false,
        controller: Some(CharacterController {
            desired_velocity: [-0.55, 0.0, 0.0],
        }),
        material: Material {
            albedo: [0.85, 0.22, 0.48],
            roughness: 0.5,
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
            emissive: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,
        },
    });
    scene
}
