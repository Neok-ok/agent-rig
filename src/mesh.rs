//! Tiny OBJ loader (v / vt / f) for triangle meshes.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub tex_indices: Vec<[u32; 3]>,
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
    })
}
