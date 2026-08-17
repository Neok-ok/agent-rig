//! Tiny mesh loader: OBJ (v / vt / f) and a constrained glTF/GLB triangle mesh.

use std::path::{Path, PathBuf};

/// glTF material alphaMode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GltfAlphaMode {
    Opaque,
    Mask,
    Blend,
}

/// pbrMetallicRoughness resolved from a glTF primitive material (if any).
#[derive(Debug, Clone)]
pub struct GltfPbrMaterial {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    /// glTF alphaMode (OPAQUE / MASK / BLEND). Default OPAQUE.
    pub alpha_mode: GltfAlphaMode,
    /// glTF alphaCutoff (MASK only; default 0.5).
    pub alpha_cutoff: f32,
    /// Sidecar image path (PNG/JPEG) next to the glTF, if the texture is a file URI.
    pub base_color_texture_path: Option<PathBuf>,
    /// Embedded image bytes (data URI or bufferView).
    pub base_color_texture_bytes: Option<Vec<u8>>,
    /// Sidecar metallic-roughness map (glTF: G=roughness, B=metallic).
    pub metallic_roughness_texture_path: Option<PathBuf>,
    /// Embedded metallic-roughness map bytes.
    pub metallic_roughness_texture_bytes: Option<Vec<u8>>,
    /// Sidecar tangent-space normal map (OpenGL, +Z up).
    pub normal_texture_path: Option<PathBuf>,
    /// Embedded tangent-space normal map bytes.
    pub normal_texture_bytes: Option<Vec<u8>>,
    /// glTF normalTexture.scale (xy multiplier, default 1).
    pub normal_scale: f32,
    /// glTF emissiveFactor (linear RGB, default [0,0,0]).
    pub emissive_factor: [f32; 3],
    /// Sidecar emissive map (sRGB RGB; multiplies the factor).
    pub emissive_texture_path: Option<PathBuf>,
    /// Embedded emissive map bytes.
    pub emissive_texture_bytes: Option<Vec<u8>>,
    /// Sidecar occlusion / AO map (glTF: R = occlusion, 0 = occluded, 1 = open).
    pub occlusion_texture_path: Option<PathBuf>,
    /// Embedded occlusion map bytes.
    pub occlusion_texture_bytes: Option<Vec<u8>>,
    /// glTF occlusionTexture.strength (default 1).
    pub occlusion_strength: f32,
    /// KHR_materials_transmission.transmissionFactor (0 = opaque, 1 = fully transmitting).
    pub transmission: f32,
    /// Index of refraction (materials.ior or KHR_materials_ior). Default 1.5.
    pub ior: f32,
    /// KHR_materials_volume.attenuationColor (linear RGB). Default [1,1,1] (no tint).
    pub attenuation_color: [f32; 3],
    /// KHR_materials_volume.attenuationDistance. Default +inf (no absorption).
    pub attenuation_distance: f32,
    /// KHR_materials_volume.thicknessFactor. Default 0.
    pub thickness: f32,
    /// KHR_materials_dispersion.dispersion (20/Abbe). Default 0 (no chromatic split).
    pub dispersion: f32,
}

impl GltfPbrMaterial {
    pub fn base_color_rgb(&self) -> [f32; 3] {
        [
            self.base_color_factor[0],
            self.base_color_factor[1],
            self.base_color_factor[2],
        ]
    }

    pub fn has_base_color_texture(&self) -> bool {
        self.base_color_texture_path.is_some() || self.base_color_texture_bytes.is_some()
    }

    pub fn has_metallic_roughness_texture(&self) -> bool {
        self.metallic_roughness_texture_path.is_some()
            || self.metallic_roughness_texture_bytes.is_some()
    }

    pub fn has_normal_texture(&self) -> bool {
        self.normal_texture_path.is_some() || self.normal_texture_bytes.is_some()
    }

    pub fn has_emissive_texture(&self) -> bool {
        self.emissive_texture_path.is_some() || self.emissive_texture_bytes.is_some()
    }

    pub fn has_occlusion_texture(&self) -> bool {
        self.occlusion_texture_path.is_some() || self.occlusion_texture_bytes.is_some()
    }

    pub fn has_transmission(&self) -> bool {
        self.transmission > 1e-4
    }

    /// Authored volume absorption (finite distance, not a hidden constant).
    pub fn has_volume_attenuation(&self) -> bool {
        self.attenuation_distance.is_finite() && self.attenuation_distance > 1e-8
    }

    /// Authored chromatic dispersion (KHR_materials_dispersion).
    pub fn has_dispersion(&self) -> bool {
        self.dispersion > 1e-6
    }

    pub fn alpha_factor(&self) -> f32 {
        self.base_color_factor[3]
    }

    /// Sample alpha = baseColorFactor[3] * texture A (glTF). No texture → factor only.
    pub fn sample_alpha(&self, u: f32, v: f32) -> Result<f32, String> {
        let factor_a = self.base_color_factor[3];
        if !self.has_base_color_texture() {
            return Ok(factor_a);
        }
        let img = if let Some(bytes) = &self.base_color_texture_bytes {
            image::load_from_memory(bytes)
                .map_err(|e| format!("load baseColor bytes: {e}"))?
                .to_rgba8()
        } else if let Some(path) = &self.base_color_texture_path {
            image::open(path)
                .map_err(|e| format!("load baseColor {path:?}: {e}"))?
                .to_rgba8()
        } else {
            return Ok(factor_a);
        };
        Ok((sample_linear_a(&img, u, v) * factor_a).clamp(0.0, 1.0))
    }

    /// Sample emissive = factor * textureRGB (glTF). No texture → factor only (often [0,0,0]).
    pub fn sample_emissive(&self, u: f32, v: f32) -> Result<[f32; 3], String> {
        if !self.has_emissive_texture() {
            return Ok(self.emissive_factor);
        }
        let img = if let Some(bytes) = &self.emissive_texture_bytes {
            image::load_from_memory(bytes)
                .map_err(|e| format!("load emissive bytes: {e}"))?
                .to_rgb8()
        } else if let Some(path) = &self.emissive_texture_path {
            image::open(path)
                .map_err(|e| format!("load emissive {path:?}: {e}"))?
                .to_rgb8()
        } else {
            return Ok(self.emissive_factor);
        };
        let (r, g, b) = sample_linear_rgb(&img, u, v);
        Ok([
            r * self.emissive_factor[0],
            g * self.emissive_factor[1],
            b * self.emissive_factor[2],
        ])
    }

    /// Sample tangent-space normal (OpenGL, +Z up). RGB unpacked as 2c-1, then normalized.
    pub fn sample_tangent_space_normal(&self, u: f32, v: f32) -> Result<[f32; 3], String> {
        if !self.has_normal_texture() {
            return Ok([0.0, 0.0, 1.0]);
        }
        let img = if let Some(bytes) = &self.normal_texture_bytes {
            image::load_from_memory(bytes)
                .map_err(|e| format!("load normal bytes: {e}"))?
                .to_rgb8()
        } else if let Some(path) = &self.normal_texture_path {
            image::open(path)
                .map_err(|e| format!("load normal {path:?}: {e}"))?
                .to_rgb8()
        } else {
            return Ok([0.0, 0.0, 1.0]);
        };
        let (r, g, b) = sample_linear_rgb(&img, u, v);
        let mut x = (2.0 * r - 1.0) * self.normal_scale;
        let mut y = (2.0 * g - 1.0) * self.normal_scale;
        let mut z = 2.0 * b - 1.0;
        let len = (x * x + y * y + z * z).sqrt();
        if len > 1e-8 {
            x /= len;
            y /= len;
            z /= len;
        }
        Ok([x, y, z])
    }

    /// Sample AO from the R channel (glTF occlusionTexture). No texture → 1.0.
    /// strength lerps toward 1: ao = 1 + strength * (R - 1).
    pub fn sample_ao(&self, u: f32, v: f32) -> Result<f32, String> {
        if !self.has_occlusion_texture() {
            return Ok(1.0);
        }
        let img = if let Some(bytes) = &self.occlusion_texture_bytes {
            image::load_from_memory(bytes)
                .map_err(|e| format!("load occlusion bytes: {e}"))?
                .to_rgb8()
        } else if let Some(path) = &self.occlusion_texture_path {
            image::open(path)
                .map_err(|e| format!("load occlusion {path:?}: {e}"))?
                .to_rgb8()
        } else {
            return Ok(1.0);
        };
        let (r, _g, _b) = sample_linear_rgb(&img, u, v);
        let ao = 1.0 + self.occlusion_strength * (r - 1.0);
        Ok(ao.clamp(0.0, 1.0))
    }

    /// Sample (metallic, roughness). Texture B/G win over scene-JSON constants;
    /// glTF metallicFactor / roughnessFactor multiply the texel (1.0 = texture is the look).
    pub fn sample_metallic_roughness(&self, u: f32, v: f32) -> Result<(f32, f32), String> {
        if !self.has_metallic_roughness_texture() {
            return Ok((self.metallic_factor, self.roughness_factor));
        }
        let img = if let Some(bytes) = &self.metallic_roughness_texture_bytes {
            image::load_from_memory(bytes)
                .map_err(|e| format!("load mr bytes: {e}"))?
                .to_rgb8()
        } else if let Some(path) = &self.metallic_roughness_texture_path {
            image::open(path)
                .map_err(|e| format!("load mr {path:?}: {e}"))?
                .to_rgb8()
        } else {
            return Ok((self.metallic_factor, self.roughness_factor));
        };
        let (r, g, b) = sample_linear_rgb(&img, u, v);
        let _ = r;
        let roughness = (g * self.roughness_factor).clamp(0.0, 1.0);
        let metallic = (b * self.metallic_factor).clamp(0.0, 1.0);
        Ok((metallic, roughness))
    }
}

fn sample_linear_a(img: &image::RgbaImage, u: f32, v: f32) -> f32 {
    let w = img.width() as f32;
    let h = img.height() as f32;
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let x = u * w - 0.5;
    let y = (1.0 - v) * h - 0.5;
    let x0 = x.floor();
    let y0 = y.floor();
    let tx = x - x0;
    let ty = y - y0;
    let p00 = alpha_at(img, x0 as i32, y0 as i32);
    let p10 = alpha_at(img, x0 as i32 + 1, y0 as i32);
    let p01 = alpha_at(img, x0 as i32, y0 as i32 + 1);
    let p11 = alpha_at(img, x0 as i32 + 1, y0 as i32 + 1);
    let a = p00 * (1.0 - tx) + p10 * tx;
    let b = p01 * (1.0 - tx) + p11 * tx;
    a * (1.0 - ty) + b * ty
}

fn alpha_at(img: &image::RgbaImage, x: i32, y: i32) -> f32 {
    let w = img.width() as i32;
    let h = img.height() as i32;
    let x = x.rem_euclid(w) as u32;
    let y = y.rem_euclid(h) as u32;
    img.get_pixel(x, y)[3] as f32 / 255.0
}

fn sample_linear_rgb(img: &image::RgbImage, u: f32, v: f32) -> (f32, f32, f32) {
    let w = img.width() as f32;
    let h = img.height() as f32;
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let x = u * w - 0.5;
    let y = (1.0 - v) * h - 0.5;
    let x0 = x.floor();
    let y0 = y.floor();
    let tx = x - x0;
    let ty = y - y0;
    let p00 = linear_at(img, x0 as i32, y0 as i32);
    let p10 = linear_at(img, x0 as i32 + 1, y0 as i32);
    let p01 = linear_at(img, x0 as i32, y0 as i32 + 1);
    let p11 = linear_at(img, x0 as i32 + 1, y0 as i32 + 1);
    let lerp = |a: [f32; 3], b: [f32; 3], t: f32| {
        [
            a[0] * (1.0 - t) + b[0] * t,
            a[1] * (1.0 - t) + b[1] * t,
            a[2] * (1.0 - t) + b[2] * t,
        ]
    };
    let a = lerp(lerp(p00, p10, tx), lerp(p01, p11, tx), ty);
    (a[0], a[1], a[2])
}

fn linear_at(img: &image::RgbImage, x: i32, y: i32) -> [f32; 3] {
    let w = img.width() as i32;
    let h = img.height() as i32;
    let x = x.rem_euclid(w) as u32;
    let y = y.rem_euclid(h) as u32;
    let p = img.get_pixel(x, y);
    [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0]
}

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub tex_indices: Vec<[u32; 3]>,
    /// Present when the source glTF primitive referenced a material.
    pub gltf_material: Option<GltfPbrMaterial>,
    /// Optional per-vertex TANGENT (xyz + handedness w). Empty if the file has none.
    pub tangents: Vec<[f32; 4]>,
}

impl TriangleMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len()
    }

    pub fn triangle_uvs(&self, tri: usize) -> [[f32; 2]; 3] {
        let ti = self.tex_indices[tri];
        [
            self.texcoords[ti[0] as usize],
            self.texcoords[ti[1] as usize],
            self.texcoords[ti[2] as usize],
        ]
    }

    pub fn geometric_normal(&self, tri: usize) -> [f32; 3] {
        let idx = self.indices[tri];
        let a = self.vertices[idx[0] as usize];
        let b = self.vertices[idx[1] as usize];
        let c = self.vertices[idx[2] as usize];
        let e1 = sub3(b, a);
        let e2 = sub3(c, a);
        norm3(cross3(e1, e2))
    }

    /// Shading normal at a barycentric hit: TBN * sampled n_ts when a normal map is present.
    pub fn shaded_normal(&self, tri: usize, bu: f32, bv: f32) -> Result<[f32; 3], String> {
        let n = self.geometric_normal(tri);
        let Some(gm) = &self.gltf_material else {
            return Ok(n);
        };
        if !gm.has_normal_texture() {
            return Ok(n);
        }
        let uvs = self.triangle_uvs(tri);
        let w = 1.0 - bu - bv;
        let uv = [
            uvs[0][0] * w + uvs[1][0] * bu + uvs[2][0] * bv,
            uvs[0][1] * w + uvs[1][1] * bu + uvs[2][1] * bv,
        ];
        let n_ts = gm.sample_tangent_space_normal(uv[0], uv[1])?;
        let (t, b, n) = self.tbn_at(tri, bu, bv, n);
        Ok(apply_tbn(t, b, n, n_ts))
    }

    pub fn tbn_at(&self, tri: usize, bu: f32, bv: f32, n: [f32; 3]) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let idx = self.indices[tri];
        if self.tangents.len() == self.vertices.len() {
            let t0 = self.tangents[idx[0] as usize];
            let t1 = self.tangents[idx[1] as usize];
            let t2 = self.tangents[idx[2] as usize];
            return tbn_from_interpolated_tangent(t0, t1, t2, bu, bv, n);
        }
        let p0 = self.vertices[idx[0] as usize];
        let p1 = self.vertices[idx[1] as usize];
        let p2 = self.vertices[idx[2] as usize];
        let uvs = self.triangle_uvs(tri);
        tbn_from_positions_uvs(p0, p1, p2, uvs[0], uvs[1], uvs[2], n)
    }

    /// Signed-tetrahedron volume (absolute). Closed meshes only.
    pub fn volume(&self) -> f32 {
        let mut vol = 0.0f32;
        for idx in &self.indices {
            let a = self.vertices[idx[0] as usize];
            let b = self.vertices[idx[1] as usize];
            let c = self.vertices[idx[2] as usize];
            vol += (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0;
        }
        vol.abs().max(1e-8)
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn mul3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm3(a: [f32; 3]) -> [f32; 3] {
    let l = dot3(a, a).sqrt();
    if l < 1e-12 {
        a
    } else {
        mul3(a, 1.0 / l)
    }
}

fn orthonormal_tangent(n: [f32; 3]) -> [f32; 3] {
    let axis = if n[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    norm3(cross3(axis, n))
}

/// TBN from triangle positions + UVs (no TANGENT accessor). OpenGL, +Y = +V.
pub fn tbn_from_positions_uvs(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    n: [f32; 3],
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let n = norm3(n);
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    let du1 = uv1[0] - uv0[0];
    let dv1 = uv1[1] - uv0[1];
    let du2 = uv2[0] - uv0[0];
    let dv2 = uv2[1] - uv0[1];
    let det = du1 * dv2 - du2 * dv1;
    let (t_raw, b_raw) = if det.abs() < 1e-8 {
        let t = orthonormal_tangent(n);
        (t, cross3(n, t))
    } else {
        let inv = 1.0 / det;
        let t = [
            (e1[0] * dv2 - e2[0] * dv1) * inv,
            (e1[1] * dv2 - e2[1] * dv1) * inv,
            (e1[2] * dv2 - e2[2] * dv1) * inv,
        ];
        let b = [
            (-e1[0] * du2 + e2[0] * du1) * inv,
            (-e1[1] * du2 + e2[1] * du1) * inv,
            (-e1[2] * du2 + e2[2] * du1) * inv,
        ];
        (t, b)
    };
    let t = norm3(sub3(t_raw, mul3(n, dot3(n, t_raw))));
    let t = if dot3(t, t) < 1e-10 {
        orthonormal_tangent(n)
    } else {
        t
    };
    let b_ortho = cross3(n, t);
    let b = if dot3(b_ortho, b_raw) < 0.0 {
        mul3(b_ortho, -1.0)
    } else {
        b_ortho
    };
    (t, norm3(b), n)
}

/// TBN from interpolated glTF TANGENT (xyz + handedness w).
pub fn tbn_from_interpolated_tangent(
    t0: [f32; 4],
    t1: [f32; 4],
    t2: [f32; 4],
    bu: f32,
    bv: f32,
    n: [f32; 3],
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let n = norm3(n);
    let w = 1.0 - bu - bv;
    let t = norm3([
        t0[0] * w + t1[0] * bu + t2[0] * bv,
        t0[1] * w + t1[1] * bu + t2[1] * bv,
        t0[2] * w + t1[2] * bu + t2[2] * bv,
    ]);
    let t = norm3(sub3(t, mul3(n, dot3(n, t))));
    let handed = if (t0[3] * w + t1[3] * bu + t2[3] * bv) < 0.0 {
        -1.0
    } else {
        1.0
    };
    let b = mul3(cross3(n, t), handed);
    (t, b, n)
}

pub fn apply_tbn(t: [f32; 3], b: [f32; 3], n: [f32; 3], n_ts: [f32; 3]) -> [f32; 3] {
    norm3(add3(add3(mul3(t, n_ts[0]), mul3(b, n_ts[1])), mul3(n, n_ts[2])))
}

pub fn resolve_mesh_path(path: &str, search_dirs: &[PathBuf]) -> Result<PathBuf, String> {
    resolve_asset_path(path, search_dirs, "mesh")
}

pub fn resolve_texture_path(path: &str, search_dirs: &[PathBuf]) -> Result<PathBuf, String> {
    resolve_asset_path(path, search_dirs, "texture")
}

pub fn resolve_asset_path(path: &str, search_dirs: &[PathBuf], kind: &str) -> Result<PathBuf, String> {
    let p = Path::new(path);
    if p.is_file() {
        return Ok(p.to_path_buf());
    }
    let mut candidates = Vec::new();
    for dir in search_dirs {
        candidates.push(dir.join(p));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(p));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(p));
    if let Some(name) = p.file_name() {
        for dir in search_dirs {
            candidates.push(dir.join(name));
        }
        candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes").join(name));
        candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("textures").join(name));
    }
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    Err(format!("{kind} not found: {path} (tried {candidates:?})"))
}

pub fn load_mesh(path: &Path) -> Result<TriangleMesh, String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "obj" => load_obj(path),
        "gltf" => load_gltf(path),
        "glb" => load_glb(path),
        other => Err(format!(
            "unsupported mesh extension '.{other}' for {path:?} (want .obj / .gltf / .glb)"
        )),
    }
}

pub fn load_obj(path: &Path) -> Result<TriangleMesh, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("read mesh {path:?}: {e}"))?;
    parse_obj(&txt).map_err(|e| format!("parse mesh {path:?}: {e}"))
}

fn resolve_index(idx: i32, len: usize) -> u32 {
    if idx < 0 {
        (len as i32 + idx) as u32
    } else {
        (idx - 1) as u32
    }
}

fn planar_xz(vertices: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut zmin = f32::MAX;
    let mut zmax = f32::MIN;
    for v in vertices {
        xmin = xmin.min(v[0]);
        xmax = xmax.max(v[0]);
        zmin = zmin.min(v[2]);
        zmax = zmax.max(v[2]);
    }
    let dx = (xmax - xmin).max(1e-6);
    let dz = (zmax - zmin).max(1e-6);
    vertices
        .iter()
        .map(|v| [(v[0] - xmin) / dx, (v[2] - zmin) / dz])
        .collect()
}

pub fn parse_obj(txt: &str) -> Result<TriangleMesh, String> {
    let mut vertices = Vec::new();
    let mut texcoords = Vec::new();
    let mut indices = Vec::new();
    let mut tex_indices = Vec::new();
    let mut have_any_vt = false;
    for (lineno, raw) in txt.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let tag = parts.next().unwrap_or("");
        match tag {
            "v" => {
                let x: f32 = parts
                    .next()
                    .ok_or_else(|| format!("line {}: missing x", lineno + 1))?
                    .parse()
                    .map_err(|e| format!("line {}: {e}", lineno + 1))?;
                let y: f32 = parts
                    .next()
                    .ok_or_else(|| format!("line {}: missing y", lineno + 1))?
                    .parse()
                    .map_err(|e| format!("line {}: {e}", lineno + 1))?;
                let z: f32 = parts
                    .next()
                    .ok_or_else(|| format!("line {}: missing z", lineno + 1))?
                    .parse()
                    .map_err(|e| format!("line {}: {e}", lineno + 1))?;
                vertices.push([x, y, z]);
            }
            "vt" => {
                let u: f32 = parts
                    .next()
                    .ok_or_else(|| format!("line {}: missing u", lineno + 1))?
                    .parse()
                    .map_err(|e| format!("line {}: {e}", lineno + 1))?;
                let v: f32 = parts
                    .next()
                    .ok_or_else(|| format!("line {}: missing v", lineno + 1))?
                    .parse()
                    .map_err(|e| format!("line {}: {e}", lineno + 1))?;
                texcoords.push([u, v]);
            }
            "f" => {
                let mut face: Vec<u32> = Vec::new();
                let mut face_vt: Vec<Option<u32>> = Vec::new();
                for tok in parts {
                    let mut segs = tok.split('/');
                    let idx_str = segs.next().unwrap_or(tok);
                    let idx: i32 = idx_str
                        .parse()
                        .map_err(|e| format!("line {}: bad index {tok}: {e}", lineno + 1))?;
                    face.push(resolve_index(idx, vertices.len()));
                    let vt = match segs.next() {
                        Some(s) if !s.is_empty() => {
                            let t: i32 = s.parse().map_err(|e| {
                                format!("line {}: bad vt {tok}: {e}", lineno + 1)
                            })?;
                            have_any_vt = true;
                            Some(resolve_index(t, texcoords.len()))
                        }
                        _ => None,
                    };
                    face_vt.push(vt);
                }
                if face.len() < 3 {
                    return Err(format!("line {}: face needs ≥3 indices", lineno + 1));
                }
                for i in 1..face.len() - 1 {
                    indices.push([face[0], face[i], face[i + 1]]);
                    tex_indices.push([
                        face_vt[0].unwrap_or(face[0]),
                        face_vt[i].unwrap_or(face[i]),
                        face_vt[i + 1].unwrap_or(face[i + 1]),
                    ]);
                }
            }
            _ => {}
        }
    }
    if vertices.is_empty() || indices.is_empty() {
        return Err("OBJ has no vertices or triangles".into());
    }
    let n = vertices.len() as u32;
    for tri in &indices {
        if tri[0] >= n || tri[1] >= n || tri[2] >= n {
            return Err(format!("face index out of range (n={n})"));
        }
    }
    if !have_any_vt || texcoords.is_empty() {
        texcoords = planar_xz(&vertices);
        tex_indices = indices.clone();
    } else {
        let nt = texcoords.len() as u32;
        for ti in &tex_indices {
            if ti[0] >= nt || ti[1] >= nt || ti[2] >= nt {
                return Err(format!("texcoord index out of range (n={nt})"));
            }
        }
    }
    Ok(TriangleMesh {
        vertices,
        indices,
        texcoords,
        tex_indices,
        gltf_material: None,
        tangents: Vec::new(),
    })
}

pub fn load_gltf(path: &Path) -> Result<TriangleMesh, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("read gltf {path:?}: {e}"))?;
    parse_gltf_json(&txt, path.parent(), None).map_err(|e| format!("parse gltf {path:?}: {e}"))
}

pub fn load_glb(path: &Path) -> Result<TriangleMesh, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read glb {path:?}: {e}"))?;
    parse_glb(&bytes, path.parent()).map_err(|e| format!("parse glb {path:?}: {e}"))
}

fn parse_glb(bytes: &[u8], base_dir: Option<&Path>) -> Result<TriangleMesh, String> {
    if bytes.len() < 12 {
        return Err("glb header too short".into());
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if magic != 0x4654_6C67 {
        return Err(format!("not a glb (magic {magic:#x})"));
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != 2 {
        return Err(format!("unsupported glb version {version}"));
    }
    let mut off = 12usize;
    let mut json: Option<String> = None;
    let mut bin: Option<Vec<u8>> = None;
    while off + 8 <= bytes.len() {
        let chunk_len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
        let chunk_type = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().unwrap());
        off += 8;
        if off + chunk_len > bytes.len() {
            return Err("glb chunk truncated".into());
        }
        let data = &bytes[off..off + chunk_len];
        match chunk_type {
            0x4E4F_534A => {
                let s = std::str::from_utf8(data).map_err(|e| format!("glb json utf8: {e}"))?;
                json = Some(s.trim_end_matches('\0').trim_end().to_string());
            }
            0x004E_4942 => bin = Some(data.to_vec()),
            _ => {}
        }
        off += chunk_len;
    }
    let json = json.ok_or_else(|| "glb missing JSON chunk".to_string())?;
    parse_gltf_json(&json, base_dir, bin.as_deref())
}

fn parse_gltf_json(
    txt: &str,
    base_dir: Option<&Path>,
    glb_bin: Option<&[u8]>,
) -> Result<TriangleMesh, String> {
    let root: serde_json::Value =
        serde_json::from_str(txt).map_err(|e| format!("gltf json: {e}"))?;
    let meshes = root
        .get("meshes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "gltf has no meshes".to_string())?;
    let prim = meshes
        .iter()
        .find_map(|m| {
            m.get("primitives")
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.first())
        })
        .ok_or_else(|| "gltf mesh has no primitives".to_string())?;
    let mode = prim.get("mode").and_then(|v| v.as_u64()).unwrap_or(4);
    if mode != 4 {
        return Err(format!("only TRIANGLES mode (4) is supported, got {mode}"));
    }
    let attrs = prim
        .get("attributes")
        .ok_or_else(|| "primitive missing attributes".to_string())?;
    let pos_acc = attrs
        .get("POSITION")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "primitive missing POSITION".to_string())? as usize;
    let uv_acc = attrs.get("TEXCOORD_0").and_then(|v| v.as_u64()).map(|v| v as usize);
    let tan_acc = attrs.get("TANGENT").and_then(|v| v.as_u64()).map(|v| v as usize);
    let idx_acc = prim.get("indices").and_then(|v| v.as_u64()).map(|v| v as usize);

    let accessors = root
        .get("accessors")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "gltf has no accessors".to_string())?;
    let views = root
        .get("bufferViews")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "gltf has no bufferViews".to_string())?;
    let buffers_meta = root
        .get("buffers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "gltf has no buffers".to_string())?;

    let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(buffers_meta.len());
    for (i, b) in buffers_meta.iter().enumerate() {
        let byte_length = b.get("byteLength").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let uri = b.get("uri").and_then(|v| v.as_str());
        buffers.push(load_gltf_buffer(uri, byte_length, base_dir, if i == 0 { glb_bin } else { None })?);
    }

    let vertices = read_vec3_accessor(accessors, views, &buffers, pos_acc)?;
    if vertices.is_empty() {
        return Err("gltf POSITION accessor is empty".into());
    }
    let texcoords = if let Some(ai) = uv_acc {
        read_vec2_accessor(accessors, views, &buffers, ai)?
    } else {
        Vec::new()
    };
    let flat_indices = if let Some(ai) = idx_acc {
        read_indices_accessor(accessors, views, &buffers, ai)?
    } else {
        (0..vertices.len() as u32).collect()
    };
    if flat_indices.len() < 3 || flat_indices.len() % 3 != 0 {
        return Err(format!(
            "gltf indices must be a multiple of 3, got {}",
            flat_indices.len()
        ));
    }
    let mut indices = Vec::with_capacity(flat_indices.len() / 3);
    for tri in flat_indices.chunks_exact(3) {
        indices.push([tri[0], tri[1], tri[2]]);
    }
    let n = vertices.len() as u32;
    for tri in &indices {
        if tri[0] >= n || tri[1] >= n || tri[2] >= n {
            return Err(format!("gltf index out of range (n={n})"));
        }
    }
    let (texcoords, tex_indices) = if texcoords.len() == vertices.len() {
        (texcoords, indices.clone())
    } else {
        (planar_xz(&vertices), indices.clone())
    };
    let tangents = if let Some(ai) = tan_acc {
        let tans = read_vec4_accessor(accessors, views, &buffers, ai)?;
        if tans.len() == vertices.len() {
            tans
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    let gltf_material = parse_primitive_material(&root, prim, base_dir, &buffers)?;
    Ok(TriangleMesh {
        vertices,
        indices,
        texcoords,
        tex_indices,
        gltf_material,
        tangents,
    })
}

fn parse_primitive_material(
    root: &serde_json::Value,
    prim: &serde_json::Value,
    base_dir: Option<&Path>,
    buffers: &[Vec<u8>],
) -> Result<Option<GltfPbrMaterial>, String> {
    let Some(mi) = prim.get("material").and_then(|v| v.as_u64()) else {
        return Ok(None);
    };
    let materials = root
        .get("materials")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "gltf primitive has material but materials[] is missing".to_string())?;
    let mat = materials
        .get(mi as usize)
        .ok_or_else(|| format!("missing gltf material {mi}"))?;
    let pbr = mat.get("pbrMetallicRoughness");
    let mut base_color_factor = [1.0f32, 1.0, 1.0, 1.0];
    let mut metallic_factor = 1.0f32;
    let mut roughness_factor = 1.0f32;
    let mut base_color_texture_path = None;
    let mut base_color_texture_bytes = None;
    let mut metallic_roughness_texture_path = None;
    let mut metallic_roughness_texture_bytes = None;
    let mut normal_texture_path = None;
    let mut normal_texture_bytes = None;
    let mut normal_scale = 1.0f32;
    let mut emissive_factor = [0.0f32, 0.0, 0.0];
    let mut emissive_texture_path = None;
    let mut emissive_texture_bytes = None;
    let mut occlusion_texture_path = None;
    let mut occlusion_texture_bytes = None;
    let mut occlusion_strength = 1.0f32;
    let mut transmission = 0.0f32;
    let mut ior = 1.5f32;
    let mut attenuation_color = [1.0f32, 1.0, 1.0];
    let mut attenuation_distance = f32::INFINITY;
    let mut thickness = 0.0f32;
    let mut dispersion = 0.0f32;
    let alpha_mode = match mat
        .get("alphaMode")
        .and_then(|v| v.as_str())
        .unwrap_or("OPAQUE")
    {
        "MASK" => GltfAlphaMode::Mask,
        "BLEND" => GltfAlphaMode::Blend,
        _ => GltfAlphaMode::Opaque,
    };
    let alpha_cutoff = mat
        .get("alphaCutoff")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5) as f32;
    if let Some(pbr) = pbr {
        if let Some(arr) = pbr.get("baseColorFactor").and_then(|v| v.as_array()) {
            for (i, v) in arr.iter().take(4).enumerate() {
                if let Some(n) = v.as_f64() {
                    base_color_factor[i] = n as f32;
                }
            }
        }
        if let Some(n) = pbr.get("metallicFactor").and_then(|v| v.as_f64()) {
            metallic_factor = n as f32;
        }
        if let Some(n) = pbr.get("roughnessFactor").and_then(|v| v.as_f64()) {
            roughness_factor = n as f32;
        }
        if let Some(tex) = pbr.get("baseColorTexture") {
            let tex_i = tex
                .get("index")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "baseColorTexture missing index".to_string())? as usize;
            let (path, bytes) = load_gltf_image(root, tex_i, base_dir, buffers)?;
            base_color_texture_path = path;
            base_color_texture_bytes = bytes;
        }
        if let Some(tex) = pbr.get("metallicRoughnessTexture") {
            let tex_i = tex
                .get("index")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "metallicRoughnessTexture missing index".to_string())?
                as usize;
            let (path, bytes) = load_gltf_image(root, tex_i, base_dir, buffers)?;
            metallic_roughness_texture_path = path;
            metallic_roughness_texture_bytes = bytes;
        }
    }
    if let Some(tex) = mat.get("normalTexture") {
        let tex_i = tex
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "normalTexture missing index".to_string())? as usize;
        let (path, bytes) = load_gltf_image(root, tex_i, base_dir, buffers)?;
        normal_texture_path = path;
        normal_texture_bytes = bytes;
        if let Some(n) = tex.get("scale").and_then(|v| v.as_f64()) {
            normal_scale = n as f32;
        }
    }
    if let Some(arr) = mat.get("emissiveFactor").and_then(|v| v.as_array()) {
        for (i, v) in arr.iter().take(3).enumerate() {
            if let Some(n) = v.as_f64() {
                emissive_factor[i] = n as f32;
            }
        }
    }
    if let Some(tex) = mat.get("emissiveTexture") {
        let tex_i = tex
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "emissiveTexture missing index".to_string())? as usize;
        let (path, bytes) = load_gltf_image(root, tex_i, base_dir, buffers)?;
        emissive_texture_path = path;
        emissive_texture_bytes = bytes;
    }
    if let Some(tex) = mat.get("occlusionTexture") {
        let tex_i = tex
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "occlusionTexture missing index".to_string())? as usize;
        let (path, bytes) = load_gltf_image(root, tex_i, base_dir, buffers)?;
        occlusion_texture_path = path;
        occlusion_texture_bytes = bytes;
        if let Some(n) = tex.get("strength").and_then(|v| v.as_f64()) {
            occlusion_strength = n as f32;
        }
    }
    if let Some(n) = mat.get("ior").and_then(|v| v.as_f64()) {
        ior = n as f32;
    }
    if let Some(ext) = mat.get("extensions") {
        if let Some(tr) = ext.get("KHR_materials_transmission") {
            if let Some(n) = tr.get("transmissionFactor").and_then(|v| v.as_f64()) {
                transmission = n as f32;
            }
        }
        if let Some(ie) = ext.get("KHR_materials_ior") {
            if let Some(n) = ie.get("ior").and_then(|v| v.as_f64()) {
                ior = n as f32;
            }
        }
        if let Some(vol) = ext.get("KHR_materials_volume") {
            if let Some(arr) = vol.get("attenuationColor").and_then(|v| v.as_array()) {
                for (i, v) in arr.iter().take(3).enumerate() {
                    if let Some(n) = v.as_f64() {
                        attenuation_color[i] = n as f32;
                    }
                }
            }
            if let Some(n) = vol.get("attenuationDistance").and_then(|v| v.as_f64()) {
                attenuation_distance = n as f32;
            }
            if let Some(n) = vol.get("thicknessFactor").and_then(|v| v.as_f64()) {
                thickness = n as f32;
            }
        }
        if let Some(disp) = ext.get("KHR_materials_dispersion") {
            if let Some(n) = disp.get("dispersion").and_then(|v| v.as_f64()) {
                dispersion = (n as f32).max(0.0);
            }
        }
    }
    Ok(Some(GltfPbrMaterial {
        base_color_factor,
        metallic_factor,
        roughness_factor,
        alpha_mode,
        alpha_cutoff,
        base_color_texture_path,
        base_color_texture_bytes,
        metallic_roughness_texture_path,
        metallic_roughness_texture_bytes,
        normal_texture_path,
        normal_texture_bytes,
        normal_scale,
        emissive_factor,
        emissive_texture_path,
        emissive_texture_bytes,
        occlusion_texture_path,
        occlusion_texture_bytes,
        occlusion_strength,
        transmission,
        ior,
        attenuation_color,
        attenuation_distance,
        thickness,
        dispersion,
    }))
}

fn load_gltf_image(
    root: &serde_json::Value,
    texture_index: usize,
    base_dir: Option<&Path>,
    buffers: &[Vec<u8>],
) -> Result<(Option<PathBuf>, Option<Vec<u8>>), String> {
    let textures = root
        .get("textures")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "gltf has a texture index but no textures[]".to_string())?;
    let tex = textures
        .get(texture_index)
        .ok_or_else(|| format!("missing texture {texture_index}"))?;
    let source = tex
        .get("source")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("texture {texture_index} missing source"))? as usize;
    let images = root
        .get("images")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "gltf has texture but no images[]".to_string())?;
    let img = images
        .get(source)
        .ok_or_else(|| format!("missing image {source}"))?;
    if let Some(uri) = img.get("uri").and_then(|v| v.as_str()) {
        if uri.starts_with("data:") {
            return Ok((None, Some(decode_data_uri(uri)?)));
        }
        let p = match base_dir {
            Some(dir) => dir.join(uri),
            None => PathBuf::from(uri),
        };
        if !p.is_file() {
            return Err(format!("gltf image not found: {p:?}"));
        }
        return Ok((Some(p), None));
    }
    if let Some(view_i) = img.get("bufferView").and_then(|v| v.as_u64()) {
        let views = root
            .get("bufferViews")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "gltf image bufferView but no bufferViews[]".to_string())?;
        let view = views
            .get(view_i as usize)
            .ok_or_else(|| format!("missing bufferView {view_i}"))?;
        let buf_i = view.get("buffer").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let buf = buffers
            .get(buf_i)
            .ok_or_else(|| format!("missing buffer {buf_i}"))?;
        let off = view.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let len = view
            .get("byteLength")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("bufferView {view_i} missing byteLength"))? as usize;
        if off + len > buf.len() {
            return Err(format!("image bufferView {view_i} out of range"));
        }
        return Ok((None, Some(buf[off..off + len].to_vec())));
    }
    Err("gltf image has neither uri nor bufferView".into())
}

fn load_gltf_buffer(
    uri: Option<&str>,
    byte_length: usize,
    base_dir: Option<&Path>,
    glb_bin: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    match uri {
        None => {
            let bin = glb_bin.ok_or_else(|| "buffer has no uri and no GLB BIN chunk".to_string())?;
            if bin.len() < byte_length {
                return Err(format!(
                    "GLB BIN is {} bytes, buffer.byteLength={byte_length}",
                    bin.len()
                ));
            }
            Ok(bin[..byte_length].to_vec())
        }
        Some(u) if u.starts_with("data:") => decode_data_uri(u),
        Some(u) => {
            let p = match base_dir {
                Some(dir) => dir.join(u),
                None => PathBuf::from(u),
            };
            std::fs::read(&p).map_err(|e| format!("read gltf buffer {p:?}: {e}"))
        }
    }
}

fn decode_data_uri(uri: &str) -> Result<Vec<u8>, String> {
    let idx = uri
        .find("base64,")
        .ok_or_else(|| "data uri is not base64".to_string())?;
    decode_base64(uri[idx + 7..].trim())
}

fn decode_base64(s: &str) -> Result<Vec<u8>, String> {
    fn val(c: u8) -> Result<u8, String> {
        Ok(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err(format!("bad base64 byte {c}")),
        })
    }
    let bytes: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' {
            break;
        }
        let a = val(bytes[i])?;
        let b = if i + 1 < bytes.len() && bytes[i + 1] != b'=' {
            val(bytes[i + 1])?
        } else {
            0
        };
        let c = if i + 2 < bytes.len() && bytes[i + 2] != b'=' {
            val(bytes[i + 2])?
        } else {
            0
        };
        let d = if i + 3 < bytes.len() && bytes[i + 3] != b'=' {
            val(bytes[i + 3])?
        } else {
            0
        };
        out.push((a << 2) | (b >> 4));
        if i + 2 < bytes.len() && bytes[i + 2] != b'=' {
            out.push((b << 4) | (c >> 2));
        }
        if i + 3 < bytes.len() && bytes[i + 3] != b'=' {
            out.push(((c & 0x03) << 6) | d);
        }
        i += 4;
    }
    Ok(out)
}

fn accessor_view<'a>(
    accessors: &'a [serde_json::Value],
    views: &'a [serde_json::Value],
    buffers: &'a [Vec<u8>],
    acc_index: usize,
) -> Result<(&'a serde_json::Value, &'a [u8], usize, usize), String> {
    let acc = accessors
        .get(acc_index)
        .ok_or_else(|| format!("missing accessor {acc_index}"))?;
    let view_i = acc
        .get("bufferView")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("accessor {acc_index} has no bufferView"))? as usize;
    let view = views
        .get(view_i)
        .ok_or_else(|| format!("missing bufferView {view_i}"))?;
    let buf_i = view.get("buffer").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let buf = buffers
        .get(buf_i)
        .ok_or_else(|| format!("missing buffer {buf_i}"))?;
    let view_off = view.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let acc_off = acc.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let view_len = view.get("byteLength").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let start = view_off + acc_off;
    if start > buf.len() || view_off + view_len > buf.len() {
        return Err(format!("bufferView {view_i} out of range"));
    }
    let end = (view_off + view_len).min(buf.len());
    let slice = &buf[start..end];
    let count = acc.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let stride = view.get("byteStride").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    Ok((acc, slice, count, stride))
}

fn read_vec3_accessor(
    accessors: &[serde_json::Value],
    views: &[serde_json::Value],
    buffers: &[Vec<u8>],
    acc_index: usize,
) -> Result<Vec<[f32; 3]>, String> {
    let (acc, data, count, stride) = accessor_view(accessors, views, buffers, acc_index)?;
    let ty = acc.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let ctype = acc.get("componentType").and_then(|v| v.as_u64()).unwrap_or(0);
    if ty != "VEC3" || ctype != 5126 {
        return Err(format!(
            "POSITION must be FLOAT VEC3, got type={ty} componentType={ctype}"
        ));
    }
    let step = if stride == 0 { 12 } else { stride };
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = i * step;
        if o + 12 > data.len() {
            return Err("POSITION buffer underrun".into());
        }
        let x = f32::from_le_bytes(data[o..o + 4].try_into().unwrap());
        let y = f32::from_le_bytes(data[o + 4..o + 8].try_into().unwrap());
        let z = f32::from_le_bytes(data[o + 8..o + 12].try_into().unwrap());
        out.push([x, y, z]);
    }
    Ok(out)
}

fn read_vec2_accessor(
    accessors: &[serde_json::Value],
    views: &[serde_json::Value],
    buffers: &[Vec<u8>],
    acc_index: usize,
) -> Result<Vec<[f32; 2]>, String> {
    let (acc, data, count, stride) = accessor_view(accessors, views, buffers, acc_index)?;
    let ty = acc.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let ctype = acc.get("componentType").and_then(|v| v.as_u64()).unwrap_or(0);
    if ty != "VEC2" || ctype != 5126 {
        return Err(format!(
            "TEXCOORD_0 must be FLOAT VEC2, got type={ty} componentType={ctype}"
        ));
    }
    let step = if stride == 0 { 8 } else { stride };
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = i * step;
        if o + 8 > data.len() {
            return Err("TEXCOORD_0 buffer underrun".into());
        }
        let u = f32::from_le_bytes(data[o..o + 4].try_into().unwrap());
        let v = f32::from_le_bytes(data[o + 4..o + 8].try_into().unwrap());
        out.push([u, v]);
    }
    Ok(out)
}

fn read_vec4_accessor(
    accessors: &[serde_json::Value],
    views: &[serde_json::Value],
    buffers: &[Vec<u8>],
    acc_index: usize,
) -> Result<Vec<[f32; 4]>, String> {
    let (acc, data, count, stride) = accessor_view(accessors, views, buffers, acc_index)?;
    let ty = acc.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let ctype = acc.get("componentType").and_then(|v| v.as_u64()).unwrap_or(0);
    if ty != "VEC4" || ctype != 5126 {
        return Err(format!(
            "TANGENT must be FLOAT VEC4, got type={ty} componentType={ctype}"
        ));
    }
    let step = if stride == 0 { 16 } else { stride };
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = i * step;
        if o + 16 > data.len() {
            return Err("TANGENT buffer underrun".into());
        }
        let x = f32::from_le_bytes(data[o..o + 4].try_into().unwrap());
        let y = f32::from_le_bytes(data[o + 4..o + 8].try_into().unwrap());
        let z = f32::from_le_bytes(data[o + 8..o + 12].try_into().unwrap());
        let w = f32::from_le_bytes(data[o + 12..o + 16].try_into().unwrap());
        out.push([x, y, z, w]);
    }
    Ok(out)
}

fn read_indices_accessor(
    accessors: &[serde_json::Value],
    views: &[serde_json::Value],
    buffers: &[Vec<u8>],
    acc_index: usize,
) -> Result<Vec<u32>, String> {
    let (acc, data, count, stride) = accessor_view(accessors, views, buffers, acc_index)?;
    let ctype = acc.get("componentType").and_then(|v| v.as_u64()).unwrap_or(0);
    let elem = match ctype {
        5121 => 1u32, // UNSIGNED_BYTE
        5123 => 2,    // UNSIGNED_SHORT
        5125 => 4,    // UNSIGNED_INT
        other => return Err(format!("unsupported index componentType {other}")),
    };
    let step = if stride == 0 { elem as usize } else { stride };
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = i * step;
        if o + elem as usize > data.len() {
            return Err("indices buffer underrun".into());
        }
        let v = match elem {
            1 => data[o] as u32,
            2 => u16::from_le_bytes(data[o..o + 2].try_into().unwrap()) as u32,
            4 => u32::from_le_bytes(data[o..o + 4].try_into().unwrap()),
            _ => unreachable!(),
        };
        out.push(v);
    }
    Ok(out)
}
