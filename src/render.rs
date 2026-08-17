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
}

/// Order-2 SH of the procedural HDR environment (y-up).
struct EnvSh {
    c: [V3; 9],
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
        });
    }

    prims
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
    let mut tmin = EPS;
    loop {
        match closest_hit(orig, dir, prims, tmin, tmax) {
            None => return false,
            Some(h) => {
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
    trace_rec(orig, dir, prims, lights, env, 0.0, MAX_BLEND_DEPTH)
}

fn trace_rec(
    orig: V3,
    dir: V3,
    prims: &[Prim],
    lights: &[Light],
    env: &EnvSh,
    tmin: f32,
    depth: u32,
) -> V3 {
    match closest_hit(orig, dir, prims, tmin, f32::MAX) {
        None => env_radiance(dir),
        Some(h) => {
            if h.alpha_mode == GltfAlphaMode::Mask && h.alpha < h.alpha_cutoff {
                return if depth == 0 {
                    env_radiance(dir)
                } else {
                    trace_rec(orig, dir, prims, lights, env, h.t + EPS, depth)
                };
            }
            let src = shade(&h, dir, prims, lights, env);
            let transmitting = h.transmission > 1e-4;
            let blend = h.alpha_mode == GltfAlphaMode::Blend && h.alpha < 0.999;
            if (transmitting || blend) && depth > 0 {
                // Increment 17: continue and composite. Increment 20: Snell-refract
                // using the authored IOR (eta = 1/ior entering, ior leaving).
                if transmitting {
                    let refr = snell_refract(dir, h.n, h.ior);
                    // Leave the hit face along the refracted direction so we
                    // traverse the slab (enter+exit) instead of re-hitting it.
                    let nudged = h.p.add(refr.mul(EPS * 4.0));
                    let behind = trace_rec(nudged, refr, prims, lights, env, 0.0, depth - 1);
                    // transmission=1 → the continuation *is* the image. Alpha
                    // tints (glass color) instead of covering the kink twice.
                    let cover = h.alpha * (1.0 - h.transmission);
                    let tint = V3::new(
                        1.0 - h.alpha * (1.0 - h.albedo.x),
                        1.0 - h.alpha * (1.0 - h.albedo.y),
                        1.0 - h.alpha * (1.0 - h.albedo.z),
                    );
                    return src.mul(cover).add(behind.hadamard(tint).mul(1.0 - cover));
                }
                let behind = trace_rec(orig, dir, prims, lights, env, h.t + EPS, depth - 1);
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
    }
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
    let f0 = V3::new(0.04, 0.04, 0.04).mul(1.0 - h.metallic).add(h.albedo.mul(h.metallic));

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

    // Specular IBL: roughness-blurred environment in the reflection direction.
    let r = n.mul(2.0 * n_dot_v).sub(v).norm();
    let spec_env = env_specular(r, n, h.roughness);
    let spec_brdf = env_brdf(n_dot_v, h.roughness, f0);
    // Contact AO keeps the 0.25 floor; sampled texture AO multiplies the whole IBL spec.
    let spec_ao = (0.25 + 0.75 * contact) * tex_ao;
    // Stronger split-sum + a little raw env sheen so horizon/sky actually paints on terracotta.
    let specular_ibl = spec_env
        .hadamard(spec_brdf)
        .mul(spec_ao * 4.4)
        .add(spec_env.mul(0.10 * spec_ao));

    let mut color = diffuse_ibl.add(specular_ibl);

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
                color = color.add(cook_torrance(
                    h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
                ));
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
                color = color.add(cook_torrance(
                    h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
                ));
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
                        acc = acc.add(cook_torrance(
                            h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
                        ));
                    }
                }
                color = color.add(acc.mul(1.0 / n_samples));
            }
        }
    }
    // Self-illumination: sampled emissive added after lighting, not a new light.
    color.add(h.emissive)
}

fn cook_torrance(albedo: V3, roughness: f32, metallic: f32, n: V3, v: V3, l: V3, n_dot_l: f32, radiance: V3) -> V3 {
    let h = v.add(l).norm();
    let n_dot_v = n.dot(v).max(1e-4);
    let n_dot_h = n.dot(h).max(0.0);
    let v_dot_h = v.dot(h).max(0.0);

    let f0 = V3::new(0.04, 0.04, 0.04).mul(1.0 - metallic).add(albedo.mul(metallic));
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
