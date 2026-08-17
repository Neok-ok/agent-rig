//! Tiny CPU Cook-Torrance raytracer with procedural IBL (spheres, oriented boxes, triangle meshes).

use image::{Rgb, RgbImage};
use std::path::Path;

use crate::mesh::{
    apply_tbn, tbn_from_interpolated_tangent, tbn_from_positions_uvs, GltfAlphaMode,
};
use crate::scene::{Light, Scene, Shape};

pub const FRAME_WIDTH: u32 = 800;
pub const FRAME_HEIGHT: u32 = 450;

const AA: u32 = 2;
const EPS: f32 = 1e-3;
const PI: f32 = std::f32::consts::PI;
const EXPOSURE: f32 = 0.74;
const SH_SAMPLES: u32 = 96;
const MAX_BLEND_DEPTH: u32 = 4;

#[derive(Clone, Copy)]
struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

impl V3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn from_arr(a: [f32; 3]) -> Self {
        Self::new(a[0], a[1], a[2])
    }
    fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
    fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
    fn hadamard(self, o: Self) -> Self {
        Self::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }
    fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    fn cross(self, o: Self) -> Self {
        Self::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    fn norm(self) -> Self {
        let l = self.len();
        if l < 1e-12 {
            self
        } else {
            self.mul(1.0 / l)
        }
    }
    fn clamp01(self) -> Self {
        Self::new(self.x.clamp(0.0, 1.0), self.y.clamp(0.0, 1.0), self.z.clamp(0.0, 1.0))
    }
}

fn lerp(a: V3, b: V3, t: f32) -> V3 {
    a.mul(1.0 - t).add(b.mul(t))
}

#[derive(Clone, Copy)]
struct Quat {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Quat {
    fn from_wxyz(a: [f32; 4]) -> Self {
        let q = Self {
            w: a[0],
            x: a[1],
            y: a[2],
            z: a[3],
        };
        let n = (q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
        if n < 1e-12 {
            Self {
                w: 1.0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            Self {
                w: q.w / n,
                x: q.x / n,
                y: q.y / n,
                z: q.z / n,
            }
        }
    }
    fn conj(self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
    fn rotate(self, v: V3) -> V3 {
        let qv = Self {
            w: 0.0,
            x: v.x,
            y: v.y,
            z: v.z,
        };
        let r = mul_q(mul_q(self, qv), self.conj());
        V3::new(r.x, r.y, r.z)
    }
}

fn mul_q(a: Quat, b: Quat) -> Quat {
    Quat {
        w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
        x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
    }
}

struct Prim {
    kind: PrimKind,
    center: V3,
    rotation: Quat,
    albedo: V3,
    roughness: f32,
    metallic: f32,
    albedo_map: Option<AlbedoMap>,
    mr_map: Option<MrMap>,
    normal_map: Option<NormalMap>,
    normal_scale: f32,
    emissive_factor: V3,
    emissive_map: Option<AlbedoMap>,
    ao_map: Option<MrMap>,
    ao_strength: f32,
    alpha: f32,
    alpha_mode: GltfAlphaMode,
    alpha_cutoff: f32,
    transmission: f32,
    ior: f32,
    attenuation_color: V3,
    attenuation_distance: f32,
    thickness: f32,
    clearcoat: f32,
    clearcoat_roughness: f32,
    sheen: f32,
    sheen_roughness: f32,
    sheen_color: V3,
    anisotropy: f32,
    anisotropy_rotation: f32,
    iridescence: f32,
    iridescence_ior: f32,
    iridescence_thickness: f32,
    dispersion: f32,
    body_index: usize,
    emissive_intensity: f32,
}

struct AlbedoMap {
    width: u32,
    height: u32,
    pixels: Vec<V3>,
    alphas: Vec<f32>,
}

impl AlbedoMap {
    fn load(path: &Path) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("load albedo {path:?}: {e}"))?
            .to_rgb8();
        Self::from_rgb8(img)
    }

    fn load_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("load albedo bytes: {e}"))?
            .to_rgb8();
        Self::from_rgb8(img)
    }

    fn from_rgb8(img: image::RgbImage) -> Result<Self, String> {
        let width = img.width();
        let height = img.height();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for px in img.pixels() {
            pixels.push(V3::new(
                srgb_u8_to_linear(px[0]),
                srgb_u8_to_linear(px[1]),
                srgb_u8_to_linear(px[2]),
            ));
        }
        let n = (width * height) as usize;
        Ok(Self {
            width,
            height,
            pixels,
            alphas: vec![1.0; n],
        })
    }

    fn load_rgba(path: &Path) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("load albedo {path:?}: {e}"))?
            .to_rgba8();
        Self::from_rgba8(img)
    }

    fn load_rgba_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("load albedo bytes: {e}"))?
            .to_rgba8();
        Self::from_rgba8(img)
    }

    fn from_rgba8(img: image::RgbaImage) -> Result<Self, String> {
        let width = img.width();
        let height = img.height();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        let mut alphas = Vec::with_capacity((width * height) as usize);
        for px in img.pixels() {
            pixels.push(V3::new(
                srgb_u8_to_linear(px[0]),
                srgb_u8_to_linear(px[1]),
                srgb_u8_to_linear(px[2]),
            ));
            alphas.push(px[3] as f32 / 255.0);
        }
        Ok(Self {
            width,
            height,
            pixels,
            alphas,
        })
    }

    fn mul_factor(&mut self, factor: V3) {
        for px in &mut self.pixels {
            *px = px.hadamard(factor);
        }
    }

    fn sample(&self, u: f32, v: f32) -> V3 {
        let w = self.width as f32;
        let h = self.height as f32;
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);
        let x = u * w - 0.5;
        let y = (1.0 - v) * h - 0.5;
        let x0 = x.floor();
        let y0 = y.floor();
        let tx = x - x0;
        let ty = y - y0;
        let p00 = self.at(x0 as i32, y0 as i32);
        let p10 = self.at(x0 as i32 + 1, y0 as i32);
        let p01 = self.at(x0 as i32, y0 as i32 + 1);
        let p11 = self.at(x0 as i32 + 1, y0 as i32 + 1);
        lerp(lerp(p00, p10, tx), lerp(p01, p11, tx), ty)
    }

    fn at(&self, x: i32, y: i32) -> V3 {
        let w = self.width as i32;
        let h = self.height as i32;
        let x = x.rem_euclid(w) as u32;
        let y = y.rem_euclid(h) as u32;
        self.pixels[(y * self.width + x) as usize]
    }

    fn sample_alpha(&self, u: f32, v: f32) -> f32 {
        if self.alphas.is_empty() {
            return 1.0;
        }
        let w = self.width as f32;
        let h = self.height as f32;
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);
        let x = u * w - 0.5;
        let y = (1.0 - v) * h - 0.5;
        let x0 = x.floor();
        let y0 = y.floor();
        let tx = x - x0;
        let ty = y - y0;
        let p00 = self.alpha_at(x0 as i32, y0 as i32);
        let p10 = self.alpha_at(x0 as i32 + 1, y0 as i32);
        let p01 = self.alpha_at(x0 as i32, y0 as i32 + 1);
        let p11 = self.alpha_at(x0 as i32 + 1, y0 as i32 + 1);
        let a = p00 * (1.0 - tx) + p10 * tx;
        let b = p01 * (1.0 - tx) + p11 * tx;
        a * (1.0 - ty) + b * ty
    }

    fn alpha_at(&self, x: i32, y: i32) -> f32 {
        let w = self.width as i32;
        let h = self.height as i32;
        let x = x.rem_euclid(w) as u32;
        let y = y.rem_euclid(h) as u32;
        self.alphas[(y * self.width + x) as usize]
    }
}

/// Linear tangent-space normal map. RGB unpacked as 2c-1 (OpenGL, +Z up).
struct NormalMap {
    width: u32,
    height: u32,
    pixels: Vec<V3>,
}

impl NormalMap {
    fn load(path: &Path) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("load normal {path:?}: {e}"))?
            .to_rgb8();
        Self::from_rgb8(img)
    }

    fn load_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("load normal bytes: {e}"))?
            .to_rgb8();
        Self::from_rgb8(img)
    }

    fn from_rgb8(img: image::RgbImage) -> Result<Self, String> {
        let width = img.width();
        let height = img.height();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for px in img.pixels() {
            pixels.push(V3::new(
                px[0] as f32 / 255.0,
                px[1] as f32 / 255.0,
                px[2] as f32 / 255.0,
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    fn sample(&self, u: f32, v: f32) -> V3 {
        let w = self.width as f32;
        let h = self.height as f32;
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);
        let x = u * w - 0.5;
        let y = (1.0 - v) * h - 0.5;
        let x0 = x.floor();
        let y0 = y.floor();
        let tx = x - x0;
        let ty = y - y0;
        let p00 = self.at(x0 as i32, y0 as i32);
        let p10 = self.at(x0 as i32 + 1, y0 as i32);
        let p01 = self.at(x0 as i32, y0 as i32 + 1);
        let p11 = self.at(x0 as i32 + 1, y0 as i32 + 1);
        lerp(lerp(p00, p10, tx), lerp(p01, p11, tx), ty)
    }

    fn at(&self, x: i32, y: i32) -> V3 {
        let w = self.width as i32;
        let h = self.height as i32;
        let x = x.rem_euclid(w) as u32;
        let y = y.rem_euclid(h) as u32;
        self.pixels[(y * self.width + x) as usize]
    }

    fn sample_ts(&self, u: f32, v: f32, scale: f32) -> V3 {
        let s = self.sample(u, v);
        V3::new((2.0 * s.x - 1.0) * scale, (2.0 * s.y - 1.0) * scale, 2.0 * s.z - 1.0).norm()
    }
}

/// Linear (non-sRGB) metallic-roughness map. glTF: G=roughness, B=metallic.
struct MrMap {
    width: u32,
    height: u32,
    pixels: Vec<V3>,
}

impl MrMap {
    fn load(path: &Path) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("load mr {path:?}: {e}"))?
            .to_rgb8();
        Self::from_rgb8(img)
    }

    fn load_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("load mr bytes: {e}"))?
            .to_rgb8();
        Self::from_rgb8(img)
    }

    fn from_rgb8(img: image::RgbImage) -> Result<Self, String> {
        let width = img.width();
        let height = img.height();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for px in img.pixels() {
            pixels.push(V3::new(
                px[0] as f32 / 255.0,
                px[1] as f32 / 255.0,
                px[2] as f32 / 255.0,
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    fn sample(&self, u: f32, v: f32) -> V3 {
        let w = self.width as f32;
        let h = self.height as f32;
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);
        let x = u * w - 0.5;
        let y = (1.0 - v) * h - 0.5;
        let x0 = x.floor();
        let y0 = y.floor();
        let tx = x - x0;
        let ty = y - y0;
        let p00 = self.at(x0 as i32, y0 as i32);
        let p10 = self.at(x0 as i32 + 1, y0 as i32);
        let p01 = self.at(x0 as i32, y0 as i32 + 1);
        let p11 = self.at(x0 as i32 + 1, y0 as i32 + 1);
        lerp(lerp(p00, p10, tx), lerp(p01, p11, tx), ty)
    }

    fn at(&self, x: i32, y: i32) -> V3 {
        let w = self.width as i32;
        let h = self.height as i32;
        let x = x.rem_euclid(w) as u32;
        let y = y.rem_euclid(h) as u32;
        self.pixels[(y * self.width + x) as usize]
    }
}

fn srgb_u8_to_linear(c: u8) -> f32 {
    let x = c as f32 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

enum PrimKind {
    Sphere { radius: f32 },
    Box { half: V3 },
    Mesh {
        tris: Vec<[V3; 3]>,
        uvs: Vec<[[f32; 2]; 3]>,
        tangents: Option<Vec<[[f32; 4]; 3]>>,
        aabb_min: V3,
        aabb_max: V3,
    },
}

struct Hit {
    t: f32,
    p: V3,
    n: V3,
    albedo: V3,
    roughness: f32,
    metallic: f32,
    emissive: V3,
    ao: f32,
    alpha: f32,
    alpha_mode: GltfAlphaMode,
    alpha_cutoff: f32,
    transmission: f32,
    ior: f32,
    attenuation_color: V3,
    attenuation_distance: f32,
    thickness: f32,
    clearcoat: f32,
    clearcoat_roughness: f32,
    sheen: f32,
    sheen_roughness: f32,
    sheen_color: V3,
    anisotropy: f32,
    anisotropy_rotation: f32,
    iridescence: f32,
    iridescence_ior: f32,
    iridescence_thickness: f32,
    dispersion: f32,
    body_index: usize,
}

/// Order-2 SH of the procedural HDR environment (y-up).
struct EnvSh {
    c: [V3; 9],
}


/// Six-face box as triangles so BLEND rays hit surfaces, not the interior.
fn trigger_box_shell(center: V3, half: V3) -> PrimKind {
    let hx = half.x;
    let hy = half.y;
    let hz = half.z;
    let c = |x: f32, y: f32, z: f32| V3::new(center.x + x, center.y + y, center.z + z);
    let faces = [
        // +X
        [c(hx, -hy, -hz), c(hx, -hy, hz), c(hx, hy, hz)],
        [c(hx, -hy, -hz), c(hx, hy, hz), c(hx, hy, -hz)],
        // -X
        [c(-hx, -hy, hz), c(-hx, -hy, -hz), c(-hx, hy, -hz)],
        [c(-hx, -hy, hz), c(-hx, hy, -hz), c(-hx, hy, hz)],
        // +Y
        [c(-hx, hy, -hz), c(hx, hy, -hz), c(hx, hy, hz)],
        [c(-hx, hy, -hz), c(hx, hy, hz), c(-hx, hy, hz)],
        // -Y
        [c(-hx, -hy, hz), c(hx, -hy, hz), c(hx, -hy, -hz)],
        [c(-hx, -hy, hz), c(hx, -hy, -hz), c(-hx, -hy, -hz)],
        // +Z
        [c(-hx, -hy, hz), c(-hx, hy, hz), c(hx, hy, hz)],
        [c(-hx, -hy, hz), c(hx, hy, hz), c(hx, -hy, hz)],
        // -Z
        [c(hx, -hy, -hz), c(hx, hy, -hz), c(-hx, hy, -hz)],
        [c(hx, -hy, -hz), c(-hx, hy, -hz), c(-hx, -hy, -hz)],
    ];
    let uv = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
    let uvs = vec![uv; faces.len()];
    let aabb_min = V3::new(center.x - hx, center.y - hy, center.z - hz);
    let aabb_max = V3::new(center.x + hx, center.y + hy, center.z + hz);
    PrimKind::Mesh {
        tris: faces.to_vec(),
        uvs,
        tangents: None,
        aabb_min,
        aabb_max,
    }
}

fn scene_prims(scene: &Scene) -> Vec<Prim> {
    let mut prims: Vec<Prim> = Vec::with_capacity(scene.bodies.len());
    for b in &scene.bodies {
        let mut albedo = V3::from_arr(b.material.albedo);
        let mut roughness = b.material.roughness.clamp(0.04, 1.0);
        let mut metallic = b.material.metallic.clamp(0.0, 1.0);
        let mut mr_map = None;
        let mut normal_map = None;
        let mut normal_scale = 1.0f32;
        let mut emissive_factor = V3::new(0.0, 0.0, 0.0);
        let mut emissive_map = None;
        let mut ao_map = None;
        let mut ao_strength = 1.0f32;
        let mut alpha = 1.0f32;
        let mut alpha_mode = GltfAlphaMode::Opaque;
        let mut alpha_cutoff = 0.5f32;
        let mut transmission = 0.0f32;
        let mut ior = 1.5f32;
        let mut attenuation_color = V3::new(1.0, 1.0, 1.0);
        let mut attenuation_distance = f32::INFINITY;
        let mut thickness = 0.0f32;
        let clearcoat = b.material.clearcoat.clamp(0.0, 1.0);
        let clearcoat_roughness = b.material.clearcoat_roughness;
        let sheen = b.material.sheen.clamp(0.0, 1.0);
        let sheen_roughness = b.material.sheen_roughness;
        let sheen_color = V3::from_arr(b.material.sheen_color);
        let anisotropy = b.material.anisotropy.clamp(0.0, 1.0);
        let anisotropy_rotation = b.material.anisotropy_rotation;
        let iridescence = b.material.iridescence.clamp(0.0, 1.0);
        let iridescence_ior = b.material.iridescence_ior;
        let iridescence_thickness = b.material.iridescence_thickness;
        let mut dispersion = b.material.dispersion.max(0.0);
        let mut albedo_map = b.material.albedo_map.as_ref().map(|p| {
            let resolved = scene
                .resolve_texture(p)
                .unwrap_or_else(|e| panic!("albedo_map {p}: {e}"));
            AlbedoMap::load(&resolved).unwrap_or_else(|e| panic!("{e}"))
        });
        let kind = match &b.shape {
            Shape::Sphere { radius } => PrimKind::Sphere { radius: *radius },
            Shape::Box { size } => PrimKind::Box {
                half: V3::new(size[0] * 0.5, size[1] * 0.5, size[2] * 0.5),
            },
            Shape::Mesh { path, .. } => {
                let mesh = scene
                    .load_body_mesh(path)
                    .unwrap_or_else(|e| panic!("load mesh {path}: {e}"));
                if let Some(gm) = &mesh.gltf_material {
                    albedo = V3::from_arr(gm.base_color_rgb());
                    roughness = gm.roughness_factor.clamp(0.04, 1.0);
                    metallic = gm.metallic_factor.clamp(0.0, 1.0);
                    albedo_map = None;
                    alpha = gm.alpha_factor().clamp(0.0, 1.0);
                    alpha_mode = gm.alpha_mode;
                    alpha_cutoff = gm.alpha_cutoff.clamp(0.0, 1.0);
                    transmission = gm.transmission.clamp(0.0, 1.0);
                    ior = gm.ior.max(1.0);
                    attenuation_color = V3::from_arr(gm.attenuation_color);
                    attenuation_distance = gm.attenuation_distance;
                    thickness = gm.thickness.max(0.0);
                    if gm.dispersion > 0.0 {
                        dispersion = gm.dispersion.max(0.0);
                    }
                    if b.material.dispersion > 0.0 {
                        dispersion = b.material.dispersion.max(0.0);
                    }
                    if let Some(bytes) = &gm.base_color_texture_bytes {
                        let mut map = AlbedoMap::load_rgba_from_bytes(bytes)
                            .unwrap_or_else(|e| panic!("gltf baseColorTexture: {e}"));
                        map.mul_factor(albedo);
                        albedo_map = Some(map);
                    } else if let Some(tex_path) = &gm.base_color_texture_path {
                        let mut map = AlbedoMap::load_rgba(tex_path)
                            .unwrap_or_else(|e| panic!("gltf baseColorTexture {tex_path:?}: {e}"));
                        map.mul_factor(albedo);
                        albedo_map = Some(map);
                    }
                    if let Some(bytes) = &gm.metallic_roughness_texture_bytes {
                        mr_map = Some(
                            MrMap::load_from_bytes(bytes)
                                .unwrap_or_else(|e| panic!("gltf metallicRoughnessTexture: {e}")),
                        );
                    } else if let Some(tex_path) = &gm.metallic_roughness_texture_path {
                        mr_map = Some(MrMap::load(tex_path).unwrap_or_else(|e| {
                            panic!("gltf metallicRoughnessTexture {tex_path:?}: {e}")
                        }));
                    }
                    if let Some(bytes) = &gm.normal_texture_bytes {
                        normal_map = Some(
                            NormalMap::load_from_bytes(bytes)
                                .unwrap_or_else(|e| panic!("gltf normalTexture: {e}")),
                        );
                        normal_scale = gm.normal_scale;
                    } else if let Some(tex_path) = &gm.normal_texture_path {
                        normal_map = Some(NormalMap::load(tex_path).unwrap_or_else(|e| {
                            panic!("gltf normalTexture {tex_path:?}: {e}")
                        }));
                        normal_scale = gm.normal_scale;
                    }
                    emissive_factor = V3::from_arr(gm.emissive_factor);
                    if let Some(bytes) = &gm.emissive_texture_bytes {
                        emissive_map = Some(
                            AlbedoMap::load_from_bytes(bytes)
                                .unwrap_or_else(|e| panic!("gltf emissiveTexture: {e}")),
                        );
                    } else if let Some(tex_path) = &gm.emissive_texture_path {
                        emissive_map = Some(AlbedoMap::load(tex_path).unwrap_or_else(|e| {
                            panic!("gltf emissiveTexture {tex_path:?}: {e}")
                        }));
                    }
                    if let Some(bytes) = &gm.occlusion_texture_bytes {
                        ao_map = Some(
                            MrMap::load_from_bytes(bytes)
                                .unwrap_or_else(|e| panic!("gltf occlusionTexture: {e}")),
                        );
                        ao_strength = gm.occlusion_strength;
                    } else if let Some(tex_path) = &gm.occlusion_texture_path {
                        ao_map = Some(MrMap::load(tex_path).unwrap_or_else(|e| {
                            panic!("gltf occlusionTexture {tex_path:?}: {e}")
                        }));
                        ao_strength = gm.occlusion_strength;
                    }
                }
                let rot = Quat::from_wxyz(b.rotation_wxyz);
                let center = V3::from_arr(b.position);
                let mut tris = Vec::with_capacity(mesh.indices.len());
                let mut uvs = Vec::with_capacity(mesh.indices.len());
                let mut tangents = if mesh.tangents.len() == mesh.vertices.len() {
                    Some(Vec::with_capacity(mesh.indices.len()))
                } else {
                    None
                };
                let mut aabb_min = V3::new(f32::MAX, f32::MAX, f32::MAX);
                let mut aabb_max = V3::new(f32::MIN, f32::MIN, f32::MIN);
                for (tri_i, idx) in mesh.indices.iter().enumerate() {
                    let a = rot
                        .rotate(V3::from_arr(mesh.vertices[idx[0] as usize]))
                        .add(center);
                    let c0 = rot
                        .rotate(V3::from_arr(mesh.vertices[idx[1] as usize]))
                        .add(center);
                    let c1 = rot
                        .rotate(V3::from_arr(mesh.vertices[idx[2] as usize]))
                        .add(center);
                    for pt in [a, c0, c1] {
                        aabb_min = V3::new(
                            aabb_min.x.min(pt.x),
                            aabb_min.y.min(pt.y),
                            aabb_min.z.min(pt.z),
                        );
                        aabb_max = V3::new(
                            aabb_max.x.max(pt.x),
                            aabb_max.y.max(pt.y),
                            aabb_max.z.max(pt.z),
                        );
                    }
                    tris.push([a, c0, c1]);
                    uvs.push(mesh.triangle_uvs(tri_i));
                    if let Some(tans) = tangents.as_mut() {
                        let rot_tan = |tan: [f32; 4]| {
                            let v = rot.rotate(V3::new(tan[0], tan[1], tan[2]));
                            [v.x, v.y, v.z, tan[3]]
                        };
                        tans.push([
                            rot_tan(mesh.tangents[idx[0] as usize]),
                            rot_tan(mesh.tangents[idx[1] as usize]),
                            rot_tan(mesh.tangents[idx[2] as usize]),
                        ]);
                    }
                }
                PrimKind::Mesh {
                    tris,
                    uvs,
                    tangents,
                    aabb_min,
                    aabb_max,
                }
            }
        };
        let emissive_intensity = b.material.emissive_intensity;
        // Intensity > 0: authored mesh-light / self-glow (emissive × intensity).
        // Intensity == 0: keep increment-16 glTF emissiveFactor × emissiveTexture.
        if emissive_intensity > 0.0 {
            emissive_factor = V3::from_arr(b.material.emissive).mul(emissive_intensity);
            emissive_map = None;
        }
        prims.push(Prim {
            kind,
            center: V3::from_arr(b.position),
            rotation: Quat::from_wxyz(b.rotation_wxyz),
            albedo,
            roughness,
            metallic,
            albedo_map,
            mr_map,
            normal_map,
            normal_scale,
            emissive_factor,
            emissive_map,
            ao_map,
            ao_strength,
            alpha,
            alpha_mode,
            alpha_cutoff,
            transmission,
            ior,
            attenuation_color,
            attenuation_distance,
            thickness,
            clearcoat,
            clearcoat_roughness,
            sheen,
            sheen_roughness,
            sheen_color,
            anisotropy,
            anisotropy_rotation,
            iridescence,
            iridescence_ior,
            iridescence_thickness,
            dispersion,
            body_index: prims.len(),
            emissive_intensity,
        });
    }

    // Authorable sensor volumes: translucent cyan boxes (BLEND), not new
    // material features on existing bodies. Sensors stay at authored pose.
    // Use a 12-triangle shell (not PrimKind::Box) so BLEND continuation
    // hits the two faces instead of EPS-stepping through the OBB volume.
    for trigger in &scene.triggers {
        let Shape::Box { size } = &trigger.shape else {
            continue;
        };
        let half = V3::new(size[0] * 0.5, size[1] * 0.5, size[2] * 0.5);
        let center = V3::from_arr(trigger.position);
        prims.push(debug_prim(
            trigger_box_shell(center, half),
            center,
            Quat::from_wxyz([1.0, 0.0, 0.0, 0.0]),
            V3::new(0.15, 0.85, 0.95),
            V3::new(0.0, 0.0, 0.0),
            0.18,
            GltfAlphaMode::Blend,
        ));
    }

    push_authored_ray_prims(scene, &mut prims);
    push_authored_sweep_prims(scene, &mut prims);

    prims
}

fn push_authored_ray_prims(scene: &Scene, prims: &mut Vec<Prim>) {
    // Authored physics rays: thin magenta segment (origin to hit, or
    // origin+dir*max_toi on a miss) plus a yellow hit marker. Hits come
    // from the dump (copied onto the scene); this is debug draw, not a
    // renderer-side physics raycast.
    for ray in &scene.raycasts {
        let origin = V3::from_arr(ray.origin);
        let dir = V3::from_arr(ray.direction).norm();
        let hit = scene.ray_hits.iter().find(|h| h.ray == ray.id);
        let end = if let Some(h) = hit {
            V3::from_arr(h.point)
        } else {
            origin.add(dir.mul(ray.max_toi))
        };
        let delta = end.sub(origin);
        let length = delta.len();
        if length > 1e-5 {
            let center = origin.add(end).mul(0.5);
            let rot = quat_from_to(V3::new(0.0, 0.0, 1.0), delta.norm());
            prims.push(debug_prim(
                PrimKind::Box {
                    half: V3::new(0.045, 0.045, length * 0.5),
                },
                center,
                rot,
                V3::new(1.0, 0.15, 0.95),
                V3::new(10.0, 0.6, 9.0),
                1.0,
                GltfAlphaMode::Opaque,
            ));
        }
        if let Some(h) = hit {
            prims.push(debug_prim(
                PrimKind::Sphere { radius: 0.07 },
                V3::from_arr(h.point),
                Quat::from_wxyz([1.0, 0.0, 0.0, 0.0]),
                V3::new(1.0, 0.92, 0.15),
                V3::new(10.0, 8.5, 0.4),
                1.0,
                GltfAlphaMode::Opaque,
            ));
        }
    }
}

fn push_authored_sweep_prims(scene: &Scene, prims: &mut Vec<Prim>) {
    // Authored physics sweeps: translucent box at the hit pose
    // (origin + dir*toi, or origin+dir*max_toi on a miss) plus a
    // small hit marker. Hits come from the dump (copied onto the
    // scene); this is debug draw, not a renderer-side shapecast.
    for sweep in &scene.shapecasts {
        let Shape::Box { size } = &sweep.shape else {
            continue;
        };
        let origin = V3::from_arr(sweep.origin);
        let dir = V3::from_arr(sweep.direction).norm();
        let hit = scene.sweep_hits.iter().find(|h| h.sweep == sweep.id);
        let toi = if let Some(h) = hit {
            h.toi
        } else {
            sweep.max_toi
        };
        let center = origin.add(dir.mul(toi));
        let half = V3::new(size[0] * 0.5, size[1] * 0.5, size[2] * 0.5);
        prims.push(debug_prim(
            PrimKind::Box { half },
            center,
            Quat::from_wxyz([1.0, 0.0, 0.0, 0.0]),
            if hit.is_some() { V3::new(0.15, 0.85, 0.95) } else { V3::new(1.0, 0.45, 0.12) },
            V3::new(0.0, 0.0, 0.0),
            0.22,
            GltfAlphaMode::Blend,
        ));
        if let Some(h) = hit {
            prims.push(debug_prim(
                PrimKind::Sphere { radius: 0.055 },
                V3::from_arr(h.point),
                Quat::from_wxyz([1.0, 0.0, 0.0, 0.0]),
                V3::new(1.0, 0.92, 0.15),
                V3::new(10.0, 8.5, 0.4),
                1.0,
                GltfAlphaMode::Opaque,
            ));
        }
    }
}

fn quat_from_to(from: V3, to: V3) -> Quat {
    let from = from.norm();
    let to = to.norm();
    let c = from.dot(to);
    if c > 0.999999 {
        return Quat::from_wxyz([1.0, 0.0, 0.0, 0.0]);
    }
    if c < -0.999999 {
        let axis = if from.x.abs() < 0.9 {
            from.cross(V3::new(1.0, 0.0, 0.0)).norm()
        } else {
            from.cross(V3::new(0.0, 1.0, 0.0)).norm()
        };
        return Quat::from_wxyz([0.0, axis.x, axis.y, axis.z]);
    }
    let axis = from.cross(to);
    Quat::from_wxyz([1.0 + c, axis.x, axis.y, axis.z])
}

fn debug_prim(
    kind: PrimKind,
    center: V3,
    rotation: Quat,
    albedo: V3,
    emissive: V3,
    alpha: f32,
    alpha_mode: GltfAlphaMode,
) -> Prim {
    Prim {
        kind,
        center,
        rotation,
        albedo,
        roughness: 0.35,
        metallic: 0.0,
        albedo_map: None,
        mr_map: None,
        normal_map: None,
        normal_scale: 1.0,
        emissive_factor: emissive,
        emissive_map: None,
        ao_map: None,
        ao_strength: 1.0,
        alpha,
        alpha_mode,
        alpha_cutoff: 0.5,
        transmission: 0.0,
        ior: 1.5,
        attenuation_color: V3::new(1.0, 1.0, 1.0),
        attenuation_distance: f32::INFINITY,
        thickness: 0.0,
        clearcoat: 0.0,
        clearcoat_roughness: 0.0,
        sheen: 0.0,
        sheen_roughness: 0.5,
        sheen_color: V3::new(1.0, 1.0, 1.0),
        anisotropy: 0.0,
        anisotropy_rotation: 0.0,
        iridescence: 0.0,
        iridescence_ior: 1.3,
        iridescence_thickness: 400.0,
        dispersion: 0.0,
        body_index: usize::MAX,
        emissive_intensity: 0.0,
    }
}

fn surface_blocks_shadow(h: &Hit) -> bool {
    match h.alpha_mode {
        GltfAlphaMode::Opaque => true,
        GltfAlphaMode::Mask => h.alpha >= h.alpha_cutoff,
        // See-through glass should not drop a hard umbra on the courtyard.
        GltfAlphaMode::Blend => false,
    }
}

fn shadow_occluded(orig: V3, dir: V3, prims: &[Prim], tmax: f32) -> bool {
    shadow_occluded_skip(orig, dir, prims, tmax, usize::MAX)
}

fn shadow_occluded_skip(orig: V3, dir: V3, prims: &[Prim], tmax: f32, skip: usize) -> bool {
    let mut tmin = EPS;
    loop {
        match closest_hit(orig, dir, prims, tmin, tmax) {
            None => return false,
            Some(h) => {
                if h.body_index == skip {
                    tmin = h.t + EPS;
                    continue;
                }
                if surface_blocks_shadow(&h) {
                    return true;
                }
                tmin = h.t + EPS;
            }
        }
    }
}

/// True if a ray from `hit_point` toward `light_pos` hits geometry before the lamp.
pub fn point_light_occluded(
    scene: &Scene,
    hit_point: [f32; 3],
    hit_normal: [f32; 3],
    light_pos: [f32; 3],
) -> bool {
    let prims = scene_prims(scene);
    let p = V3::from_arr(hit_point);
    let n = V3::from_arr(hit_normal).norm();
    let to_l = V3::from_arr(light_pos).sub(p);
    let dist = to_l.len().max(1e-3);
    let l = to_l.mul(1.0 / dist);
    let orig = p.add(n.mul(EPS * 4.0));
    shadow_occluded(orig, l, &prims, dist)
}

/// Orthonormal width/height axes for a rectangle with the given world normal.
/// Width prefers +X when the panel faces down.
fn area_axes(n: V3) -> (V3, V3) {
    let n = n.norm();
    let helper = if n.x.abs() < 0.9 {
        V3::new(1.0, 0.0, 0.0)
    } else {
        V3::new(0.0, 1.0, 0.0)
    };
    let u = helper.sub(n.mul(helper.dot(n))).norm();
    let v = n.cross(u).norm();
    (u, v)
}

const AREA_SAMPLES_X: u32 = 4;
const AREA_SAMPLES_Y: u32 = 4;

fn area_sample_point(center: V3, u_axis: V3, v_axis: V3, size: [f32; 2], ix: u32, iy: u32) -> V3 {
    let su = ((ix as f32 + 0.5) / AREA_SAMPLES_X as f32 - 0.5) * size[0];
    let sv = ((iy as f32 + 0.5) / AREA_SAMPLES_Y as f32 - 0.5) * size[1];
    center.add(u_axis.mul(su)).add(v_axis.mul(sv))
}

/// Fraction of area-light samples visible from `hit_point` (0 = umbra, 1 = fully lit).
/// Softness is the authored rectangle `size`, sampled on a 4×4 grid.
pub fn area_light_visibility(
    scene: &Scene,
    hit_point: [f32; 3],
    hit_normal: [f32; 3],
    light_pos: [f32; 3],
    size: [f32; 2],
    light_normal: [f32; 3],
) -> f32 {
    let prims = scene_prims(scene);
    let p = V3::from_arr(hit_point);
    let n = V3::from_arr(hit_normal).norm();
    let center = V3::from_arr(light_pos);
    let n_l = V3::from_arr(light_normal).norm();
    let (u_axis, v_axis) = area_axes(n_l);
    let orig = p.add(n.mul(EPS * 4.0));
    let mut seen = 0u32;
    let total = AREA_SAMPLES_X * AREA_SAMPLES_Y;
    for iy in 0..AREA_SAMPLES_Y {
        for ix in 0..AREA_SAMPLES_X {
            let sample = area_sample_point(center, u_axis, v_axis, size, ix, iy);
            let to_l = sample.sub(p);
            let dist = to_l.len().max(1e-3);
            let l = to_l.mul(1.0 / dist);
            if n_l.dot(l.mul(-1.0)) <= 0.0 {
                continue;
            }
            if !shadow_occluded(orig, l, &prims, dist) {
                seen += 1;
            }
        }
    }
    seen as f32 / total as f32
}

pub fn render_scene(scene: &Scene, width: u32, height: u32) -> RgbImage {
    let prims = scene_prims(scene);

    let env = project_env_sh();

    let cam_pos = V3::from_arr(scene.camera.position);
    let look = V3::from_arr(scene.camera.look_at);
    let fwd = look.sub(cam_pos).norm();
    let world_up = V3::new(0.0, 1.0, 0.0);
    let mut right = fwd.cross(world_up);
    if right.len() < 1e-6 {
        right = fwd.cross(V3::new(0.0, 0.0, 1.0));
    }
    let right = right.norm();
    let up = right.cross(fwd).norm();
    let aspect = width as f32 / height as f32;
    let fov = scene.camera.fov_y_deg.to_radians();
    let half_h = (fov * 0.5).tan();
    let half_w = half_h * aspect;

    let mut img = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let mut acc = V3::new(0.0, 0.0, 0.0);
            for ay in 0..AA {
                for ax in 0..AA {
                    let jx = (x as f32 + (ax as f32 + 0.5) / AA as f32) / width as f32;
                    let jy = (y as f32 + (ay as f32 + 0.5) / AA as f32) / height as f32;
                    let sx = (2.0 * jx - 1.0) * half_w;
                    let sy = (1.0 - 2.0 * jy) * half_h;
                    let dir = fwd.add(right.mul(sx)).add(up.mul(sy)).norm();
                    acc = acc.add(trace(cam_pos, dir, &prims, &scene.lights, &env));
                }
            }
            acc = acc.mul(1.0 / (AA * AA) as f32);
            let rgb = to_srgb(tonemap(acc));
            img.put_pixel(x, y, Rgb(rgb));
        }
    }
    img
}

pub fn render_scene_to_png(scene: &Scene, width: u32, height: u32, path: &Path) -> Result<(), String> {
    let img = render_scene(scene, width, height);
    img.save(path).map_err(|e| format!("write png {path:?}: {e}"))
}

fn trace(orig: V3, dir: V3, prims: &[Prim], lights: &[Light], env: &EnvSh) -> V3 {
    trace_rec(orig, dir, prims, lights, env, 0.0, MAX_BLEND_DEPTH, None)
}

/// Cauchy IOR for a wavelength in micrometres.
/// n(λ) = ior + dispersion * (1/λ² - 1/0.55²). Green (0.55 µm) stays at `ior`.
fn cauchy_ior(ior: f32, dispersion: f32, lambda_um: f32) -> f32 {
    let n = ior + dispersion * (1.0 / (lambda_um * lambda_um) - 1.0 / (0.55 * 0.55));
    n.max(1.0)
}

fn transmit_continue(
    h: &Hit,
    dir: V3,
    src: V3,
    ior: f32,
    prims: &[Prim],
    lights: &[Light],
    env: &EnvSh,
    depth: u32,
    ior_override: Option<f32>,
) -> V3 {
    let refr = snell_refract(dir, h.n, ior);
    // Leave the hit face along the refracted direction so we
    // traverse the slab (enter+exit) instead of re-hitting it.
    let nudged = h.p.add(refr.mul(EPS * 4.0));
    let behind = trace_rec(nudged, refr, prims, lights, env, 0.0, depth - 1, ior_override);
    // transmission=1 → the continuation *is* the image. Alpha
    // tints (glass color) instead of covering the kink twice.
    let cover = h.alpha * (1.0 - h.transmission);
    let volume = h.attenuation_distance.is_finite() && h.attenuation_distance > 1e-8;
    // Entering: incident points against the outward normal.
    let entering = dir.dot(h.n) <= 0.0;
    let tint = if volume {
        // Volume absorption replaces the surface albedo tint.
        // Apply Beer-Lambert once on the enter → exit segment (per-ray).
        if entering {
            beer_lambert_through(h, nudged, refr, prims)
        } else {
            V3::new(1.0, 1.0, 1.0)
        }
    } else {
        V3::new(
            1.0 - h.alpha * (1.0 - h.albedo.x),
            1.0 - h.alpha * (1.0 - h.albedo.y),
            1.0 - h.alpha * (1.0 - h.albedo.z),
        )
    };
    src.mul(cover).add(behind.hadamard(tint).mul(1.0 - cover))
}

fn trace_rec(
    orig: V3,
    dir: V3,
    prims: &[Prim],
    lights: &[Light],
    env: &EnvSh,
    tmin: f32,
    depth: u32,
    ior_override: Option<f32>,
) -> V3 {
    match closest_hit(orig, dir, prims, tmin, f32::MAX) {
        None => env_radiance(dir),
        Some(h) => {
            if h.alpha_mode == GltfAlphaMode::Mask && h.alpha < h.alpha_cutoff {
                return if depth == 0 {
                    env_radiance(dir)
                } else {
                    trace_rec(orig, dir, prims, lights, env, h.t + EPS, depth, ior_override)
                };
            }
            let src = shade(&h, dir, prims, lights, env);
            let transmitting = h.transmission > 1e-4;
            let blend = h.alpha_mode == GltfAlphaMode::Blend && h.alpha < 0.999;
            if (transmitting || blend) && depth > 0 {
                // Increment 17: continue and composite. Increment 20: Snell-refract
                // using the authored IOR (eta = 1/ior entering, ior leaving).
                // Increment 27: when dispersion > 0, split R/G/B with Cauchy IOR.
                if transmitting {
                    let disp = h.dispersion;
                    if disp > 1e-6 && ior_override.is_none() {
                        let r = transmit_continue(
                            &h, dir, src, cauchy_ior(h.ior, disp, 0.65),
                            prims, lights, env, depth, Some(cauchy_ior(h.ior, disp, 0.65)),
                        );
                        let g = transmit_continue(
                            &h, dir, src, cauchy_ior(h.ior, disp, 0.55),
                            prims, lights, env, depth, Some(cauchy_ior(h.ior, disp, 0.55)),
                        );
                        let b = transmit_continue(
                            &h, dir, src, cauchy_ior(h.ior, disp, 0.45),
                            prims, lights, env, depth, Some(cauchy_ior(h.ior, disp, 0.45)),
                        );
                        return V3::new(r.x, g.y, b.z);
                    }
                    let ior = ior_override.unwrap_or(h.ior);
                    return transmit_continue(&h, dir, src, ior, prims, lights, env, depth, ior_override);
                }
                let behind = trace_rec(orig, dir, prims, lights, env, h.t + EPS, depth - 1, ior_override);
                return src.mul(h.alpha).add(behind.mul(1.0 - h.alpha));
            }
            src
        }
    }
}

fn closest_hit(orig: V3, dir: V3, prims: &[Prim], tmin: f32, tmax: f32) -> Option<Hit> {
    let mut best: Option<Hit> = None;
    let mut tmax = tmax;
    for p in prims {
        if let Some(h) = intersect(orig, dir, p, tmin, tmax) {
            tmax = h.t;
            best = Some(h);
        }
    }
    best
}

fn intersect(orig: V3, dir: V3, p: &Prim, tmin: f32, tmax: f32) -> Option<Hit> {
    match &p.kind {
        PrimKind::Sphere { radius } => intersect_sphere(orig, dir, p, *radius, tmin, tmax),
        PrimKind::Box { half } => intersect_obb(orig, dir, p, *half, tmin, tmax),
        PrimKind::Mesh {
            tris,
            uvs,
            tangents,
            aabb_min,
            aabb_max,
        } => intersect_mesh(
            orig, dir, p, tris, uvs, tangents.as_deref(), *aabb_min, *aabb_max, tmin, tmax,
        ),
    }
}

fn intersect_aabb(orig: V3, dir: V3, mn: V3, mx: V3, tmin: f32, tmax: f32) -> bool {
    let mut t0 = tmin;
    let mut t1 = tmax;
    for (o, d, a, b) in [
        (orig.x, dir.x, mn.x, mx.x),
        (orig.y, dir.y, mn.y, mx.y),
        (orig.z, dir.z, mn.z, mx.z),
    ] {
        if d.abs() < 1e-12 {
            if o < a || o > b {
                return false;
            }
            continue;
        }
        let inv = 1.0 / d;
        let mut tn = (a - o) * inv;
        let mut tf = (b - o) * inv;
        if tn > tf {
            std::mem::swap(&mut tn, &mut tf);
        }
        t0 = t0.max(tn);
        t1 = t1.min(tf);
        if t0 > t1 {
            return false;
        }
    }
    true
}

/// Möller–Trumbore. Faceted face normal + barycentric (u,v) of v1,v2.
fn intersect_triangle(
    orig: V3,
    dir: V3,
    v0: V3,
    v1: V3,
    v2: V3,
    tmin: f32,
    tmax: f32,
) -> Option<(f32, V3, f32, f32)> {
    let e1 = v1.sub(v0);
    let e2 = v2.sub(v0);
    let pvec = dir.cross(e2);
    let det = e1.dot(pvec);
    if det.abs() < 1e-8 {
        return None;
    }
    let inv = 1.0 / det;
    let tvec = orig.sub(v0);
    let u = tvec.dot(pvec) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let qvec = tvec.cross(e1);
    let v = dir.dot(qvec) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(qvec) * inv;
    if t < tmin || t > tmax {
        return None;
    }
    Some((t, e1.cross(e2).norm(), u, v))
}

fn intersect_mesh(
    orig: V3,
    dir: V3,
    p: &Prim,
    tris: &[[V3; 3]],
    uvs: &[[[f32; 2]; 3]],
    tangents: Option<&[[[f32; 4]; 3]]>,
    aabb_min: V3,
    aabb_max: V3,
    tmin: f32,
    tmax: f32,
) -> Option<Hit> {
    if !intersect_aabb(orig, dir, aabb_min, aabb_max, tmin, tmax) {
        return None;
    }
    let mut best_t = tmax;
    let mut best: Option<(f32, V3, V3, [f32; 2])> = None;
    for (i, ([v0, v1, v2], tuv)) in tris.iter().zip(uvs.iter()).enumerate() {
        if let Some((t, n, bu, bv)) = intersect_triangle(orig, dir, *v0, *v1, *v2, tmin, best_t) {
            best_t = t;
            let w = 1.0 - bu - bv;
            let uv = [
                tuv[0][0] * w + tuv[1][0] * bu + tuv[2][0] * bv,
                tuv[0][1] * w + tuv[1][1] * bu + tuv[2][1] * bv,
            ];
            let n = perturb_mesh_normal(n, uv, *v0, *v1, *v2, *tuv, bu, bv, tangents.and_then(|ts| ts.get(i)), p);
            best = Some((t, orig.add(dir.mul(t)), n, uv));
        }
    }
    best.map(|(t, pnt, n, uv)| hit_uv(t, pnt, n, p, Some(uv)))
}

fn v3_to_arr(v: V3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn arr_to_v3(a: [f32; 3]) -> V3 {
    V3::new(a[0], a[1], a[2])
}

fn perturb_mesh_normal(
    geom_n: V3,
    uv: [f32; 2],
    v0: V3,
    v1: V3,
    v2: V3,
    tuv: [[f32; 2]; 3],
    bu: f32,
    bv: f32,
    tangents: Option<&[[f32; 4]; 3]>,
    p: &Prim,
) -> V3 {
    let Some(nmap) = &p.normal_map else {
        return geom_n;
    };
    let n_ts = nmap.sample_ts(uv[0], uv[1], p.normal_scale);
    let n = v3_to_arr(geom_n);
    let (t, b, n) = if let Some(ts) = tangents {
        tbn_from_interpolated_tangent(ts[0], ts[1], ts[2], bu, bv, n)
    } else {
        tbn_from_positions_uvs(v3_to_arr(v0), v3_to_arr(v1), v3_to_arr(v2), tuv[0], tuv[1], tuv[2], n)
    };
    arr_to_v3(apply_tbn(t, b, n, [n_ts.x, n_ts.y, n_ts.z]))
}

fn intersect_sphere(orig: V3, dir: V3, p: &Prim, radius: f32, tmin: f32, tmax: f32) -> Option<Hit> {
    let oc = orig.sub(p.center);
    let a = dir.dot(dir);
    let b = 2.0 * oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let s = disc.sqrt();
    let mut t = (-b - s) / (2.0 * a);
    if t < tmin || t > tmax {
        t = (-b + s) / (2.0 * a);
        if t < tmin || t > tmax {
            return None;
        }
    }
    let hit_p = orig.add(dir.mul(t));
    let n = hit_p.sub(p.center).norm();
    Some(hit(t, hit_p, n, p))
}

fn intersect_obb(orig: V3, dir: V3, p: &Prim, half: V3, tmin: f32, tmax: f32) -> Option<Hit> {
    let inv = p.rotation.conj();
    let o = inv.rotate(orig.sub(p.center));
    let d = inv.rotate(dir);
    let mut t0 = tmin;
    let mut t1 = tmax;
    let mut n_local = V3::new(0.0, 1.0, 0.0);

    let axes = [
        (o.x, d.x, half.x, V3::new(1.0, 0.0, 0.0)),
        (o.y, d.y, half.y, V3::new(0.0, 1.0, 0.0)),
        (o.z, d.z, half.z, V3::new(0.0, 0.0, 1.0)),
    ];
    for (ao, ad, ah, axis) in axes {
        if ad.abs() < 1e-8 {
            if ao.abs() > ah {
                return None;
            }
            continue;
        }
        let inv_d = 1.0 / ad;
        let mut t_near = (-ah - ao) * inv_d;
        let mut t_far = (ah - ao) * inv_d;
        let mut n_near = axis.mul(-1.0);
        if t_near > t_far {
            std::mem::swap(&mut t_near, &mut t_far);
            n_near = axis;
        }
        if t_near > t0 {
            t0 = t_near;
            n_local = n_near;
        }
        t1 = t1.min(t_far);
        if t0 > t1 {
            return None;
        }
    }
    if t0 < tmin || t0 > tmax {
        return None;
    }
    let hit_p = orig.add(dir.mul(t0));
    let n = p.rotation.rotate(n_local).norm();
    Some(hit(t0, hit_p, n, p))
}

fn hit(t: f32, pnt: V3, n: V3, prim: &Prim) -> Hit {
    hit_uv(t, pnt, n, prim, None)
}

fn hit_uv(t: f32, pnt: V3, n: V3, prim: &Prim, uv: Option<[f32; 2]>) -> Hit {
    let albedo = match (&prim.albedo_map, uv) {
        (Some(map), Some([u, v])) => map.sample(u, v),
        _ => prim.albedo,
    };
    let (metallic, roughness) = match (&prim.mr_map, uv) {
        (Some(map), Some([u, v])) => {
            let s = map.sample(u, v);
            // glTF: roughness = G * roughnessFactor, metallic = B * metallicFactor.
            let roughness = (s.y * prim.roughness).clamp(0.04, 1.0);
            let metallic = (s.z * prim.metallic).clamp(0.0, 1.0);
            (metallic, roughness)
        }
        _ => (prim.metallic, prim.roughness),
    };
    let emissive = match (&prim.emissive_map, uv) {
        (Some(map), Some([u, v])) => map.sample(u, v).hadamard(prim.emissive_factor),
        _ => prim.emissive_factor,
    };
    let ao = match (&prim.ao_map, uv) {
        (Some(map), Some([u, v])) => {
            let r = map.sample(u, v).x;
            (1.0 + prim.ao_strength * (r - 1.0)).clamp(0.0, 1.0)
        }
        _ => 1.0,
    };
    let tex_a = match (&prim.albedo_map, uv) {
        (Some(map), Some([u, v])) => map.sample_alpha(u, v),
        _ => 1.0,
    };
    let alpha = (prim.alpha * tex_a).clamp(0.0, 1.0);
    Hit {
        t,
        p: pnt,
        n,
        albedo,
        roughness,
        metallic,
        emissive,
        ao,
        alpha,
        alpha_mode: prim.alpha_mode,
        alpha_cutoff: prim.alpha_cutoff,
        transmission: prim.transmission,
        ior: prim.ior,
        attenuation_color: prim.attenuation_color,
        attenuation_distance: prim.attenuation_distance,
        thickness: prim.thickness,
        clearcoat: prim.clearcoat,
        clearcoat_roughness: prim.clearcoat_roughness,
        sheen: prim.sheen,
        sheen_roughness: prim.sheen_roughness,
        sheen_color: prim.sheen_color,
        anisotropy: prim.anisotropy,
        anisotropy_rotation: prim.anisotropy_rotation,
        iridescence: prim.iridescence,
        iridescence_ior: prim.iridescence_ior,
        iridescence_thickness: prim.iridescence_thickness,
        dispersion: prim.dispersion,
        body_index: prim.body_index,
    }
}

/// Beer-Lambert transmittance for a path of `distance` through a volume:
/// T = attenuationColor.pow(distance / attenuationDistance) per channel.
fn beer_lambert(color: V3, att_dist: f32, distance: f32) -> V3 {
    if !att_dist.is_finite() || att_dist <= 1e-8 || distance <= 0.0 {
        return V3::new(1.0, 1.0, 1.0);
    }
    let t = distance / att_dist;
    V3::new(
        color.x.max(0.0).powf(t),
        color.y.max(0.0).powf(t),
        color.z.max(0.0).powf(t),
    )
}

/// Path-length transmittance from an enter hit to the exit (or authored thickness).
fn beer_lambert_through(enter: &Hit, orig: V3, dir: V3, prims: &[Prim]) -> V3 {
    let distance = if let Some(exit) = closest_hit(orig, dir, prims, 0.0, f32::MAX) {
        if exit.transmission > 1e-4 {
            // Next hit is the far face of the volume: use the real path length.
            exit.p.sub(enter.p).len()
        } else if enter.thickness > 1e-6 {
            enter.thickness
        } else {
            0.0
        }
    } else if enter.thickness > 1e-6 {
        enter.thickness
    } else {
        0.0
    };
    beer_lambert(enter.attenuation_color, enter.attenuation_distance, distance)
}

/// Snell's law. `incident` points toward the surface; `n` is the geometric normal.
/// Entering (air → material): eta = 1/ior. Leaving: eta = ior. TIR reflects.
fn snell_refract(incident: V3, n: V3, ior: f32) -> V3 {
    let ior = ior.max(1e-4);
    let mut nl = n;
    let mut eta = 1.0 / ior;
    if incident.dot(n) > 0.0 {
        nl = n.mul(-1.0);
        eta = ior;
    }
    let cosi = (-incident.dot(nl)).clamp(0.0, 1.0);
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        return incident.sub(nl.mul(2.0 * incident.dot(nl))).norm();
    }
    incident.mul(eta).add(nl.mul(eta * cosi - k.sqrt())).norm()
}

fn shade(h: &Hit, view_dir: V3, prims: &[Prim], lights: &[Light], env: &EnvSh) -> V3 {
    let n = if h.n.dot(view_dir.mul(-1.0)) < 0.0 {
        h.n.mul(-1.0)
    } else {
        h.n
    };
    let v = view_dir.mul(-1.0).norm();
    let n_dot_v = n.dot(v).max(1e-4);
    let mut f0 = V3::new(0.04, 0.04, 0.04).mul(1.0 - h.metallic).add(h.albedo.mul(h.metallic));
    if h.iridescence > 1e-4 {
        f0 = apply_iridescence_f0(
            f0,
            n_dot_v,
            h.iridescence,
            h.iridescence_ior,
            h.iridescence_thickness,
        );
    }

    let contact = contact_ao(h.p, n, prims);
    let tex_ao = h.ao.clamp(0.0, 1.0);
    let ao = (contact * tex_ao).clamp(0.0, 1.0);

    // Diffuse IBL: SH fill + explicit sky/ground by N.y so the ball Y-gradient stays readable.
    let ir_sh = sh_irradiance(env, n);
    let hemi_t = (n.y.clamp(-1.0, 1.0) * 0.5 + 0.5).powf(1.25);
    let sky_irr = env_radiance(V3::new(0.0, 1.0, 0.0)).mul(0.62);
    let gnd_irr = env_radiance(V3::new(0.0, -1.0, 0.0)).mul(0.38);
    let irradiance = ir_sh.mul(0.18).add(lerp(gnd_irr, sky_irr, hemi_t));
    let f_diff = fresnel_schlick(n_dot_v, f0);
    let k_d = V3::new(1.0 - f_diff.x, 1.0 - f_diff.y, 1.0 - f_diff.z).mul(1.0 - h.metallic);
    let diffuse_ibl = k_d.hadamard(h.albedo).hadamard(irradiance).mul(ao);

    let (tan_aniso, bit_aniso) = anisotropy_frame(n, h.anisotropy_rotation);
    // Specular IBL: roughness-blurred environment in the reflection direction.
    // Anisotropic materials use a bent reflection + stretched env cone so the
    // brushed streak reads. Strength/direction come from authored fields.
    let r = n.mul(2.0 * n_dot_v).sub(v).norm();
    let spec_env = if h.anisotropy > 1e-4 {
        let r_ani = bent_aniso_reflection(n, v, tan_aniso, h.anisotropy);
        env_specular_aniso(r_ani, tan_aniso, bit_aniso, h.roughness, h.anisotropy)
    } else {
        env_specular(r, n, h.roughness)
    };
    let spec_brdf = env_brdf(n_dot_v, h.roughness, f0);
    // Contact AO keeps the 0.25 floor; sampled texture AO multiplies the whole IBL spec.
    let spec_ao = (0.25 + 0.75 * contact) * tex_ao;
    // Stronger split-sum + a little raw env sheen so horizon/sky actually paints on terracotta.
    let irid_tint = if h.iridescence > 1e-4 {
        lerp(
            V3::new(1.0, 1.0, 1.0),
            saturate_iridescence_hue(iridescence_hue(
                n_dot_v,
                h.iridescence_ior,
                h.iridescence_thickness,
            )),
            h.iridescence,
        )
    } else {
        V3::new(1.0, 1.0, 1.0)
    };
    let specular_ibl = spec_env
        .hadamard(spec_brdf)
        .mul(spec_ao * 4.4)
        .add(spec_env.hadamard(irid_tint).mul(0.10 * spec_ao));

    let mut color = diffuse_ibl.add(specular_ibl);
    if h.iridescence > 1e-4 {
        let hue = saturate_iridescence_hue(iridescence_hue(
            n_dot_v,
            h.iridescence_ior,
            h.iridescence_thickness,
        ));
        // Extra thin-film spec on IBL so the brushed streak is rainbow, not gold.
        color = color.add(spec_env.hadamard(hue).mul(h.iridescence * spec_ao * 3.2));
    }

    // Anisotropic IBL lobe: evaluate the real GGX against the env sun and
    // along the authored tangent so the env streak is the lobe itself.
    if h.anisotropy > 1e-4 {
        let sun = V3::new(0.45, 1.0, 0.35).norm();
        let a = h.anisotropy;
        let dirs = [
            sun,
            r.add(tan_aniso.mul(a)).norm(),
            r.add(tan_aniso.mul(-a)).norm(),
            r.add(tan_aniso.mul(0.5 * a)).norm(),
            r.add(tan_aniso.mul(-0.5 * a)).norm(),
        ];
        for dir in dirs {
            let n_dot_l = n.dot(dir).max(0.0);
            if n_dot_l <= 0.0 {
                continue;
            }
            let rad = env_radiance(dir).mul(spec_ao);
            color = color.add(cook_torrance_aniso(
                h.albedo,
                h.roughness,
                h.metallic,
                n,
                v,
                dir,
                n_dot_l,
                rad,
                tan_aniso,
                bit_aniso,
                h.anisotropy,
                h.iridescence,
                h.iridescence_ior,
                h.iridescence_thickness,
            ));
            if h.clearcoat > 1e-4 {
                color = color.add(clearcoat_specular(
                    h.clearcoat_roughness.clamp(0.04, 1.0),
                    n,
                    v,
                    dir,
                    n_dot_l,
                    rad,
                    h.clearcoat,
                    tan_aniso,
                    bit_aniso,
                    h.anisotropy,
                    h.iridescence,
                    h.iridescence_ior,
                    h.iridescence_thickness,
                ));
            }
        }
    }

    // Extra dielectric clearcoat IBL (F0 ≈ 0.04). Softness is authored
    // clearcoat_roughness, not a hidden constant. On top of base MR, not a tweak.
    if h.clearcoat > 1e-4 {
        let cc_w = h.clearcoat.clamp(0.0, 1.0);
        let cc_rough = h.clearcoat_roughness.clamp(0.04, 1.0);
        let mut cc_f0 = V3::new(0.04, 0.04, 0.04);
        if h.iridescence > 1e-4 {
            cc_f0 = apply_iridescence_f0(
                cc_f0,
                n_dot_v,
                h.iridescence,
                h.iridescence_ior,
                h.iridescence_thickness,
            );
        }
        let cc_env = if h.anisotropy > 1e-4 {
            let r_ani = bent_aniso_reflection(n, v, tan_aniso, h.anisotropy);
            env_specular_aniso(r_ani, tan_aniso, bit_aniso, cc_rough, h.anisotropy)
        } else {
            env_specular(r, n, cc_rough)
        };
        let cc_brdf = env_brdf(n_dot_v, cc_rough, cc_f0);
        let cc_tint = if h.iridescence > 1e-4 { irid_tint } else { V3::new(1.0, 1.0, 1.0) };
        color = color.add(
            cc_env
                .hadamard(cc_brdf)
                .hadamard(cc_tint)
                .mul(cc_w * spec_ao * 11.0)
                .add(cc_env.hadamard(cc_tint).mul(1.45 * spec_ao * cc_w)),
        );
    }

    // Extra fabric/velvet sheen IBL. Color = authored sheen_color, softness =
    // authored sheen_roughness, weight = authored sheen. Grazing Charlie-like
    // rim (high 1-N·V), not a base-albedo tint of the whole surface.
    // Env contributes luminance only so the tint stays the authored crimson.
    if h.sheen > 1e-4 {
        let sh_w = h.sheen.clamp(0.0, 1.0);
        let sh_rough = h.sheen_roughness.clamp(0.04, 1.0);
        let sh_col = h.sheen_color;
        let grazing = (1.0 - n_dot_v).clamp(0.0, 1.0);
        // Higher authored roughness → lower power → broader / softer rim.
        let power = 0.55 + 2.4 * (1.0 - sh_rough);
        let rim = grazing.powf(power);
        let sh_env = env_specular(r, n, sh_rough);
        let tangent_v = v.sub(n.mul(n_dot_v));
        let wrap_dir = if tangent_v.len() > 1e-5 {
            tangent_v.norm()
        } else {
            r
        };
        let wrap = env_radiance(wrap_dir);
        let env_lum = (0.2126 * sh_env.x + 0.7152 * sh_env.y + 0.0722 * sh_env.z).max(0.0);
        let wrap_lum = (0.2126 * wrap.x + 0.7152 * wrap.y + 0.0722 * wrap.z).max(0.0);
        // Grazing mix: velvet fibers take over the rim/top. Not a dye of the
        // whole bench — facing pixels keep the authored albedo.
        let take = (sh_w * rim * 0.82).clamp(0.0, 0.88);
        let sheen_ibl = sh_col.mul(
            sh_w * rim * spec_ao * (1.10 * env_lum + 1.40 * wrap_lum) * 2.4,
        );
        color = color.mul(1.0 - take).add(sheen_ibl);
    }

    for light in lights {
        match light {
            Light::Directional {
                direction,
                color: lcol,
                intensity,
            } => {
                let ldir = V3::from_arr(*direction).norm();
                let l = ldir.mul(-1.0);
                let n_dot_l = n.dot(l).max(0.0);
                if n_dot_l <= 0.0 {
                    continue;
                }
                let shadow_orig = h.p.add(n.mul(EPS * 4.0));
                if shadow_occluded(shadow_orig, l, prims, f32::MAX) {
                    continue;
                }
                // Keep the sun, but do not let intensity 3 nuke the IBL Y-gradient.
                let radiance = V3::from_arr(*lcol).mul(*intensity * 0.58);
                color = color.add(base_specular(
                    h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
                    tan_aniso, bit_aniso, h.anisotropy,
                    h.iridescence, h.iridescence_ior, h.iridescence_thickness,
                ));
                if h.clearcoat > 1e-4 {
                    color = color.add(clearcoat_specular(
                        h.clearcoat_roughness.clamp(0.04, 1.0),
                        n,
                        v,
                        l,
                        n_dot_l,
                        radiance,
                        h.clearcoat,
                        tan_aniso,
                        bit_aniso,
                        h.anisotropy,
                        h.iridescence,
                        h.iridescence_ior,
                        h.iridescence_thickness,
                    ));
                }
                if h.sheen > 1e-4 {
                    color = color.add(sheen_lobe(
                        h.sheen_color,
                        h.sheen_roughness.clamp(0.04, 1.0),
                        n,
                        v,
                        l,
                        n_dot_l,
                        radiance,
                        h.sheen,
                    ));
                }
            }
            Light::Point {
                position,
                color: lcol,
                intensity,
            } => {
                let to_l = V3::from_arr(*position).sub(h.p);
                let dist = to_l.len().max(1e-3);
                let l = to_l.mul(1.0 / dist);
                let n_dot_l = n.dot(l).max(0.0);
                if n_dot_l <= 0.0 {
                    continue;
                }
                // Inverse-square falloff; occlude if anything sits between the hit and the lamp.
                let shadow_orig = h.p.add(n.mul(EPS * 4.0));
                if shadow_occluded(shadow_orig, l, prims, dist) {
                    continue;
                }
                let atten = 1.0 / (dist * dist);
                let radiance = V3::from_arr(*lcol).mul(*intensity * atten);
                color = color.add(base_specular(
                    h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
                    tan_aniso, bit_aniso, h.anisotropy,
                    h.iridescence, h.iridescence_ior, h.iridescence_thickness,
                ));
                if h.clearcoat > 1e-4 {
                    color = color.add(clearcoat_specular(
                        h.clearcoat_roughness.clamp(0.04, 1.0),
                        n,
                        v,
                        l,
                        n_dot_l,
                        radiance,
                        h.clearcoat,
                        tan_aniso,
                        bit_aniso,
                        h.anisotropy,
                        h.iridescence,
                        h.iridescence_ior,
                        h.iridescence_thickness,
                    ));
                }
                if h.sheen > 1e-4 {
                    color = color.add(sheen_lobe(
                        h.sheen_color,
                        h.sheen_roughness.clamp(0.04, 1.0),
                        n,
                        v,
                        l,
                        n_dot_l,
                        radiance,
                        h.sheen,
                    ));
                }
            }
            Light::Area {
                position,
                size,
                color: lcol,
                intensity,
                normal: lnorm,
            } => {
                let center = V3::from_arr(*position);
                let n_l = V3::from_arr(*lnorm).norm();
                let (u_axis, v_axis) = area_axes(n_l);
                let n_samples = (AREA_SAMPLES_X * AREA_SAMPLES_Y) as f32;
                let mut acc = V3::new(0.0, 0.0, 0.0);
                let shadow_orig = h.p.add(n.mul(EPS * 4.0));
                for iy in 0..AREA_SAMPLES_Y {
                    for ix in 0..AREA_SAMPLES_X {
                        let sample = area_sample_point(center, u_axis, v_axis, *size, ix, iy);
                        let to_l = sample.sub(h.p);
                        let dist = to_l.len().max(1e-3);
                        let l = to_l.mul(1.0 / dist);
                        let n_dot_l = n.dot(l).max(0.0);
                        if n_dot_l <= 0.0 {
                            continue;
                        }
                        // One-sided panel: only the facing side contributes.
                        if n_l.dot(l.mul(-1.0)) <= 0.0 {
                            continue;
                        }
                        if shadow_occluded(shadow_orig, l, prims, dist) {
                            continue;
                        }
                        let atten = 1.0 / (dist * dist);
                        let radiance = V3::from_arr(*lcol).mul(*intensity * atten);
                        acc = acc.add(base_specular(
                            h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
                            tan_aniso, bit_aniso, h.anisotropy,
                            h.iridescence, h.iridescence_ior, h.iridescence_thickness,
                        ));
                        if h.clearcoat > 1e-4 {
                            acc = acc.add(clearcoat_specular(
                                h.clearcoat_roughness.clamp(0.04, 1.0),
                                n,
                                v,
                                l,
                                n_dot_l,
                                radiance,
                                h.clearcoat,
                                tan_aniso,
                                bit_aniso,
                                h.anisotropy,
                                h.iridescence,
                                h.iridescence_ior,
                                h.iridescence_thickness,
                            ));
                        }
                        if h.sheen > 1e-4 {
                            acc = acc.add(sheen_lobe(
                                h.sheen_color,
                                h.sheen_roughness.clamp(0.04, 1.0),
                                n,
                                v,
                                l,
                                n_dot_l,
                                radiance,
                                h.sheen,
                            ));
                        }
                    }
                }
                color = color.add(acc.mul(1.0 / n_samples));
            }
        }
    }
    // Mesh lights: bodies with emissive_intensity > 0 act as a point light at
    // their post-step COM. Not pushed into scene.lights. Skip the emitter
    // itself (self-glow is the emissive add below) and skip it in shadows
    // so the lantern sphere does not umbra the courtyard.
    for prim in prims {
        if prim.emissive_intensity <= 0.0 {
            continue;
        }
        if prim.body_index == h.body_index {
            continue;
        }
        let to_l = prim.center.sub(h.p);
        let dist = to_l.len().max(1e-3);
        let l = to_l.mul(1.0 / dist);
        let n_dot_l = n.dot(l).max(0.0);
        if n_dot_l <= 0.0 {
            continue;
        }
        let shadow_orig = h.p.add(n.mul(EPS * 4.0));
        if shadow_occluded_skip(shadow_orig, l, prims, dist, prim.body_index) {
            continue;
        }
        let atten = 1.0 / (dist * dist + 1e-4);
        // emissive_factor is already emissive × intensity when intensity > 0.
        let radiance = prim.emissive_factor.mul(atten);
        color = color.add(base_specular(
            h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
            tan_aniso, bit_aniso, h.anisotropy,
            h.iridescence, h.iridescence_ior, h.iridescence_thickness,
        ));
        if h.clearcoat > 1e-4 {
            color = color.add(clearcoat_specular(
                h.clearcoat_roughness.clamp(0.04, 1.0),
                n,
                v,
                l,
                n_dot_l,
                radiance,
                h.clearcoat,
                tan_aniso,
                bit_aniso,
                h.anisotropy,
                h.iridescence,
                h.iridescence_ior,
                h.iridescence_thickness,
            ));
        }
        if h.sheen > 1e-4 {
            color = color.add(sheen_lobe(
                h.sheen_color,
                h.sheen_roughness.clamp(0.04, 1.0),
                n,
                v,
                l,
                n_dot_l,
                radiance,
                h.sheen,
            ));
        }
    }
    // Self-illumination: sampled emissive added after lighting, not a new light.
    // Intensity > 0: authored emissive × intensity. Intensity 0: increment-16
    // emissiveFactor × emissiveTexture (unscaled).
    color.add(h.emissive)
}

/// Extra dielectric coat lobe (F0 ≈ 0.04). Weight is authored `clearcoat`.
/// Roughness is authored `clearcoat_roughness` (clamped). Not a base-MR tweak.
fn clearcoat_specular(
    roughness: f32,
    n: V3,
    v: V3,
    l: V3,
    n_dot_l: f32,
    radiance: V3,
    weight: f32,
    t: V3,
    b: V3,
    anisotropy: f32,
    iridescence: f32,
    iridescence_ior: f32,
    iridescence_thickness: f32,
) -> V3 {
    let h = v.add(l).norm();
    let n_dot_v = n.dot(v).max(1e-4);
    let v_dot_h = v.dot(h).max(0.0);
    let mut f0 = V3::new(0.04, 0.04, 0.04);
    if iridescence > 1e-4 {
        f0 = apply_iridescence_f0(
            f0, v_dot_h, iridescence, iridescence_ior, iridescence_thickness,
        );
    }
    let f = fresnel_schlick(v_dot_h, f0);
    let (d, g) = if anisotropy > 1e-4 {
        let (at, ab) = aniso_alphas(roughness, anisotropy);
        (
            ggx_d_aniso(h, n, t, b, at, ab),
            smith_g1_aniso(v, n, t, b, at, ab) * smith_g1_aniso(l, n, t, b, at, ab),
        )
    } else {
        let n_dot_h = n.dot(h).max(0.0);
        (ggx_d(n_dot_h, roughness), geometry_smith(n_dot_v, n_dot_l, roughness))
    };
    let spec = f.mul(d * g / (4.0 * n_dot_v * n_dot_l + 1e-4));
    spec.hadamard(radiance).mul(n_dot_l * weight)
}

/// Extra fabric/velvet Charlie sheen lobe. Color = authored `sheen_color`,
/// roughness = authored `sheen_roughness` (softer/broader when higher),
/// weight = authored `sheen`. Intensity rises toward grazing (high 1-N·V).
/// Extra layer, not a base-albedo tint.
fn sheen_lobe(
    sheen_color: V3,
    roughness: f32,
    n: V3,
    v: V3,
    l: V3,
    n_dot_l: f32,
    radiance: V3,
    weight: f32,
) -> V3 {
    let h = v.add(l).norm();
    let n_dot_v = n.dot(v).max(1e-4);
    let n_dot_h = n.dot(h).max(0.0);
    let d = charlie_d(n_dot_h, roughness);
    let vis = ashikhmin_v(n_dot_l, n_dot_v);
    let grazing = (1.0 - n_dot_v).clamp(0.0, 1.0);
    // Softness from authored roughness: higher → broader (lower power).
    let power = 0.55 + 2.4 * (1.0 - roughness);
    let rim = grazing.powf(power);
    // Rim-weighted so the lobe reads as velvet sheen, not a facing dye.
    let light_lum = (0.2126 * radiance.x + 0.7152 * radiance.y + 0.0722 * radiance.z).max(0.0);
    sheen_color.mul(d * vis * n_dot_l * weight * (0.18 + 3.6 * rim) * light_lum)
}

/// Charlie NDF (Estevez & Kulla). Roughness is the authored sheen_roughness.
fn charlie_d(n_dot_h: f32, roughness: f32) -> f32 {
    let inv_a = 1.0 / roughness.max(1e-4);
    let sin2h = (1.0 - n_dot_h * n_dot_h).max(0.0078125);
    (2.0 + inv_a) * sin2h.powf(inv_a * 0.5) / (2.0 * PI)
}

/// Ashikhmin visibility used with Charlie sheen.
fn ashikhmin_v(n_dot_l: f32, n_dot_v: f32) -> f32 {
    (1.0 / (4.0 * (n_dot_l + n_dot_v - n_dot_l * n_dot_v))).clamp(0.0, 1.0)
}

fn base_specular(
    albedo: V3,
    roughness: f32,
    metallic: f32,
    n: V3,
    v: V3,
    l: V3,
    n_dot_l: f32,
    radiance: V3,
    t: V3,
    b: V3,
    anisotropy: f32,
    iridescence: f32,
    iridescence_ior: f32,
    iridescence_thickness: f32,
) -> V3 {
    if anisotropy > 1e-4 {
        cook_torrance_aniso(
            albedo, roughness, metallic, n, v, l, n_dot_l, radiance, t, b, anisotropy,
            iridescence, iridescence_ior, iridescence_thickness,
        )
    } else {
        cook_torrance(
            albedo, roughness, metallic, n, v, l, n_dot_l, radiance,
            iridescence, iridescence_ior, iridescence_thickness,
        )
    }
}

/// Stable tangent/bitangent on a sphere: up × N, then rotate around N by
/// the authored anisotropy_rotation (radians).
fn anisotropy_frame(n: V3, rotation: f32) -> (V3, V3) {
    let up = if n.y.abs() < 0.9 {
        V3::new(0.0, 1.0, 0.0)
    } else {
        V3::new(1.0, 0.0, 0.0)
    };
    let t0 = up.cross(n).norm();
    let b0 = n.cross(t0).norm();
    let c = rotation.cos();
    let s = rotation.sin();
    let t = t0.mul(c).add(b0.mul(s)).norm();
    let b = n.cross(t).norm();
    (t, b)
}

/// Filament/Three.js bent-normal reflection: mix N toward the anisotropic
/// normal so the env lookup stretches along the tangent.
fn bent_aniso_reflection(n: V3, v: V3, t: V3, anisotropy: f32) -> V3 {
    let aniso_t = t.cross(v);
    let bent_n = if aniso_t.len() > 1e-6 {
        let aniso_n = aniso_t.norm().cross(t).norm();
        lerp(n, aniso_n, anisotropy).norm()
    } else {
        n
    };
    let ndv = bent_n.dot(v).max(1e-4);
    bent_n.mul(2.0 * ndv).sub(v).norm()
}

/// Anisotropic env cone: wider along T, tighter along B. Stretch from
/// authored anisotropy (`at = roughness*(1+a)`, `ab = roughness*(1-a)`).
/// Authored stretch: at = roughness * (1 + anisotropy),
/// ab = roughness * (1 - anisotropy). Strength is the authored field.
fn aniso_alphas(roughness: f32, anisotropy: f32) -> (f32, f32) {
    // Standard stretch in GGX-alpha space: α = roughness², then ± authored anisotropy.
    let alpha = (roughness * roughness).max(1e-5);
    let at = (alpha * (1.0 + anisotropy)).max(1e-5);
    let ab = (alpha * (1.0 - anisotropy)).max(1e-5);
    (at, ab)
}

fn env_specular_aniso(r: V3, t: V3, b: V3, roughness: f32, anisotropy: f32) -> V3 {
    // Cone in roughness space so the env streak length follows authored anisotropy.
    let ab = (roughness * (1.0 - anisotropy)).clamp(0.01, 1.0);
    let r = r.norm();
    let mut t_p = t.sub(r.mul(t.dot(r)));
    let mut b_p = b.sub(r.mul(b.dot(r)));
    t_p = if t_p.len() > 1e-5 { t_p.norm() } else { t };
    b_p = if b_p.len() > 1e-5 { b_p.norm() } else { b };
    // Streak length follows authored anisotropy (offset along T).
    let kt = anisotropy.max(1e-4);
    let kb = ab.max(1e-4);
    let mut acc = env_radiance(r);
    acc = acc.add(env_radiance(r.add(t_p.mul(kt)).norm()));
    acc = acc.add(env_radiance(r.add(t_p.mul(-kt)).norm()));
    acc = acc.add(env_radiance(r.add(t_p.mul(2.0 * kt)).norm()));
    acc = acc.add(env_radiance(r.add(t_p.mul(-2.0 * kt)).norm()));
    acc = acc.add(env_radiance(r.add(b_p.mul(kb)).norm()));
    acc = acc.add(env_radiance(r.add(b_p.mul(-kb)).norm()));
    acc.mul(1.0 / 7.0)
}

/// Burley/Kulla anisotropic GGX NDF.
fn ggx_d_aniso(h: V3, n: V3, t: V3, b: V3, at: f32, ab: f32) -> f32 {
    let ht = h.dot(t) / at;
    let hb = h.dot(b) / ab;
    let hn = h.dot(n);
    let d = ht * ht + hb * hb + hn * hn;
    1.0 / (PI * at * ab * d * d + 1e-7)
}

/// Heitz anisotropic Smith G1.
fn smith_g1_aniso(v: V3, n: V3, t: V3, b: V3, at: f32, ab: f32) -> f32 {
    let n_dot_v = n.dot(v).abs().max(1e-4);
    let ax = at * v.dot(t);
    let ay = ab * v.dot(b);
    let lambda = (ax * ax + ay * ay) / (n_dot_v * n_dot_v);
    2.0 / (1.0 + (1.0 + lambda).sqrt())
}

/// Cook-Torrance with an anisotropic GGX lobe. `at/ab` stretch from the
/// authored anisotropy; tangent direction from authored anisotropy_rotation.
fn cook_torrance_aniso(
    albedo: V3,
    roughness: f32,
    metallic: f32,
    n: V3,
    v: V3,
    l: V3,
    n_dot_l: f32,
    radiance: V3,
    t: V3,
    b: V3,
    anisotropy: f32,
    iridescence: f32,
    iridescence_ior: f32,
    iridescence_thickness: f32,
) -> V3 {
    let h = v.add(l).norm();
    let n_dot_v = n.dot(v).max(1e-4);
    let v_dot_h = v.dot(h).max(0.0);
    let (at, ab) = aniso_alphas(roughness, anisotropy);
    let mut f0 = V3::new(0.04, 0.04, 0.04)
        .mul(1.0 - metallic)
        .add(albedo.mul(metallic));
    if iridescence > 1e-4 {
        f0 = apply_iridescence_f0(
            f0, v_dot_h, iridescence, iridescence_ior, iridescence_thickness,
        );
    }
    let f = fresnel_schlick(v_dot_h, f0);
    let d = ggx_d_aniso(h, n, t, b, at, ab);
    let g = smith_g1_aniso(v, n, t, b, at, ab) * smith_g1_aniso(l, n, t, b, at, ab);
    let spec = f.mul(d * g / (4.0 * n_dot_v * n_dot_l + 1e-4));
    let k_d = V3::new(1.0 - f.x, 1.0 - f.y, 1.0 - f.z).mul(1.0 - metallic);
    let diffuse = albedo.mul(1.0 / PI);
    k_d.hadamard(diffuse).add(spec).hadamard(radiance).mul(n_dot_l)
}

fn cook_torrance(
    albedo: V3,
    roughness: f32,
    metallic: f32,
    n: V3,
    v: V3,
    l: V3,
    n_dot_l: f32,
    radiance: V3,
    iridescence: f32,
    iridescence_ior: f32,
    iridescence_thickness: f32,
) -> V3 {
    let h = v.add(l).norm();
    let n_dot_v = n.dot(v).max(1e-4);
    let n_dot_h = n.dot(h).max(0.0);
    let v_dot_h = v.dot(h).max(0.0);

    let mut f0 = V3::new(0.04, 0.04, 0.04).mul(1.0 - metallic).add(albedo.mul(metallic));
    if iridescence > 1e-4 {
        f0 = apply_iridescence_f0(
            f0, v_dot_h, iridescence, iridescence_ior, iridescence_thickness,
        );
    }
    let f = fresnel_schlick(v_dot_h, f0);
    let d = ggx_d(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, roughness);
    let spec = f.mul(d * g / (4.0 * n_dot_v * n_dot_l + 1e-4));
    let k_d = V3::new(1.0 - f.x, 1.0 - f.y, 1.0 - f.z).mul(1.0 - metallic);
    let diffuse = albedo.mul(1.0 / PI);
    k_d.hadamard(diffuse).add(spec).hadamard(radiance).mul(n_dot_l)
}

fn ggx_d(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    a2 / (PI * d * d + 1e-7)
}

fn geometry_schlick_ggx(n_dot_x: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    n_dot_x / (n_dot_x * (1.0 - k) + k)
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    geometry_schlick_ggx(n_dot_v, roughness) * geometry_schlick_ggx(n_dot_l, roughness)
}

/// View-dependent thin-film hue from optical path 2 * n * d * cos(θ).
/// `n` and `d` (nm) are the authored iridescence_ior and iridescence_thickness.
fn iridescence_hue(cos_theta: f32, n: f32, thickness_nm: f32) -> V3 {
    let n = n.max(1.0001);
    let ct = cos_theta.clamp(0.0, 1.0);
    // Snell into the film (air → film) so IOR actually bends the path.
    let sin2_t = (1.0 - ct * ct) / (n * n);
    let cos_t = (1.0 - sin2_t).max(0.0).sqrt();
    let opd = 2.0 * n * thickness_nm * cos_t;
    let sample = |lambda: f32| -> f32 {
        let phase = 2.0 * PI * opd / lambda;
        0.5 + 0.5 * phase.cos()
    };
    // CIE-ish RGB peaks so 300–800 nm films sweep cyan / magenta / green.
    V3::new(sample(650.0), sample(530.0), sample(450.0))
}

fn saturate_iridescence_hue(hue: V3) -> V3 {
    let avg = (hue.x + hue.y + hue.z) * (1.0 / 3.0);
    V3::new(
        ((hue.x - avg) * 3.2 + 0.5).clamp(0.0, 1.0),
        ((hue.y - avg) * 3.2 + 0.5).clamp(0.0, 1.0),
        ((hue.z - avg) * 3.2 + 0.5).clamp(0.0, 1.0),
    )
}

/// Tint specular F0 with the thin-film hue. Factor / IOR / thickness are authored.
/// Extra layer on the metal Fresnel — not a base-albedo dye.
fn apply_iridescence_f0(f0: V3, cos_theta: f32, factor: f32, ior: f32, thickness_nm: f32) -> V3 {
    if factor <= 1e-4 {
        return f0;
    }
    let sat = saturate_iridescence_hue(iridescence_hue(cos_theta, ior, thickness_nm));
    // Mix F0 toward the spectral hue. Authored factor is the mix; do not
    // re-anchor to gold luminance (that kept the highlight gold-only).
    lerp(f0, sat, factor.clamp(0.0, 1.0))
}

fn fresnel_schlick(cos_theta: f32, f0: V3) -> V3 {
    let f = (1.0 - cos_theta).clamp(0.0, 1.0).powf(5.0);
    V3::new(
        f0.x + (1.0 - f0.x) * f,
        f0.y + (1.0 - f0.y) * f,
        f0.z + (1.0 - f0.z) * f,
    )
}

/// Soft contact AO: two short rays (normal + sky-ish) so the ground catches the ball.
fn contact_ao(p: V3, n: V3, prims: &[Prim]) -> f32 {
    let orig = p.add(n.mul(EPS * 4.0));
    let a_n = ao_ray(orig, n, prims, 2.5);
    let skyish = n.add(V3::new(0.0, 1.0, 0.0)).norm();
    let a_s = ao_ray(orig, skyish, prims, 2.5);
    0.4 * a_n + 0.6 * a_s
}

fn ao_ray(orig: V3, dir: V3, prims: &[Prim], max_dist: f32) -> f32 {
    let mut tmin = EPS;
    loop {
        match closest_hit(orig, dir, prims, tmin, max_dist) {
            None => return 1.0,
            Some(h) => {
                if surface_blocks_shadow(&h) {
                    let t = (h.t / max_dist).clamp(0.0, 1.0);
                    return 0.10 + 0.90 * t.sqrt();
                }
                tmin = h.t + EPS;
            }
        }
    }
}

/// HDR gradient sky + tight horizon band + soft sun aureole (linear radiance).
fn env_radiance(dir: V3) -> V3 {
    let d = dir.norm();
    let y = d.y;
    // High-contrast: saturated cool sky vs dark ground (readable Y-gradient on a small ball).
    let ground = V3::new(0.016, 0.014, 0.012);
    let horizon = V3::new(0.72, 0.92, 1.35);
    let zenith = V3::new(0.14, 0.38, 1.95);

    let base = if y <= 0.0 {
        // Ground takes over quickly so the lower hemisphere is not a second sky.
        let t = (-y).clamp(0.0, 1.0).powf(0.22);
        lerp(horizon, ground, t)
    } else {
        let t = y.clamp(0.0, 1.0).powf(0.40);
        lerp(horizon, zenith, t)
    };

    // Bright thin horizon — the glossy env cue on terracotta.
    let hz = (-y * y * 14.0).exp();
    let glow = V3::new(1.55, 1.35, 1.05).mul(0.55 * hz);

    let sun = V3::new(0.45, 1.0, 0.35).norm();
    let m = d.dot(sun).max(0.0);
    let aureole = m.powf(56.0) * 2.8 + m.powf(10.0) * 0.28;
    base.add(glow).add(V3::new(1.0, 0.88, 0.68).mul(aureole))
}

fn sh_y(dir: V3) -> [f32; 9] {
    let x = dir.x;
    let y = dir.y;
    let z = dir.z;
    [
        0.282095,
        0.488603 * y,
        0.488603 * z,
        0.488603 * x,
        1.092548 * x * z,
        1.092548 * y * z,
        0.315392 * (3.0 * y * y - 1.0),
        1.092548 * x * y,
        0.546274 * (x * x - z * z),
    ]
}

fn project_env_sh() -> EnvSh {
    let mut c = [V3::new(0.0, 0.0, 0.0); 9];
    let n = SH_SAMPLES;
    let golden = PI * (3.0 - 5.0_f32.sqrt());
    let w = 4.0 * PI / n as f32;
    for i in 0..n {
        let y = 1.0 - (i as f32 + 0.5) / n as f32 * 2.0;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let theta = golden * i as f32;
        let dir = V3::new(theta.cos() * r, y, theta.sin() * r);
        let l = env_radiance(dir);
        let ylm = sh_y(dir);
        for k in 0..9 {
            c[k] = c[k].add(l.mul(ylm[k] * w));
        }
    }
    EnvSh { c }
}

/// Cosine-convolved irradiance / π (ready to multiply by albedo).
fn sh_irradiance(env: &EnvSh, n: V3) -> V3 {
    let ylm = sh_y(n.norm());
    // A0/π = 1, A1/π = 2/3, A2/π = 1/4
    let a = [1.0, 2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 0.25, 0.25, 0.25, 0.25, 0.25];
    let mut e = V3::new(0.0, 0.0, 0.0);
    for k in 0..9 {
        e = e.add(env.c[k].mul(ylm[k] * a[k]));
    }
    V3::new(e.x.max(0.0), e.y.max(0.0), e.z.max(0.0)).mul(1.05)
}

/// Roughness cone around R (4 env taps). No mid-mip wash — keep horizon/sky readable.
fn env_specular(r: V3, _n: V3, roughness: f32) -> V3 {
    let a = roughness.clamp(0.04, 1.0);
    // Tight cone at r=0.35 so we do not average toward a gray mid color.
    let cone = a * 0.22;
    let r = r.norm();
    let up = if r.y.abs() < 0.9 {
        V3::new(0.0, 1.0, 0.0)
    } else {
        V3::new(1.0, 0.0, 0.0)
    };
    let t = r.cross(up).norm();
    let b = t.cross(r).norm();
    let ring = cone.max(1e-4);
    let mut acc = env_radiance(r);
    acc = acc.add(env_radiance(r.add(t.mul(ring)).norm()));
    acc = acc.add(env_radiance(r.add(t.mul(-0.5 * ring)).add(b.mul(0.866 * ring)).norm()));
    acc = acc.add(env_radiance(r.add(t.mul(-0.5 * ring)).add(b.mul(-0.866 * ring)).norm()));
    acc.mul(0.25)
}

/// UE4 split-sum envBRDF fit (Karis).
fn env_brdf(n_dot_v: f32, roughness: f32, f0: V3) -> V3 {
    let c0 = [-1.0, -0.0275, -0.572, 0.022];
    let c1 = [1.0, 0.0425, 1.04, -0.04];
    let rx = roughness * c0[0] + c1[0];
    let ry = roughness * c0[1] + c1[1];
    let rz = roughness * c0[2] + c1[2];
    let rw = roughness * c0[3] + c1[3];
    let a004 = (rx * rx).min((-9.28 * n_dot_v).exp2()) * rx + ry;
    let a = -1.04 * a004 + rz;
    let b = 1.04 * a004 + rw;
    f0.mul(a).add(V3::new(b, b, b))
}

fn tonemap(c: V3) -> V3 {
    fn map(x: f32) -> f32 {
        let a = 2.51;
        let b = 0.03;
        let c = 2.43;
        let d = 0.59;
        let e = 0.14;
        ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0)
    }
    let exposed = c.mul(EXPOSURE);
    V3::new(map(exposed.x), map(exposed.y), map(exposed.z)).clamp01()
}

fn to_srgb(c: V3) -> [u8; 3] {
    fn enc(x: f32) -> u8 {
        let s = if x <= 0.0031308 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        };
        (s.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    }
    [enc(c.x), enc(c.y), enc(c.z)]
}
