//! Tiny OBJ loader (v / f) for triangle meshes.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}

impl TriangleMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len()
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
    }
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    Err(format!("mesh not found: {path} (tried {candidates:?})"))
}

pub fn load_obj(path: &Path) -> Result<TriangleMesh, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("read mesh {path:?}: {e}"))?;
    parse_obj(&txt).map_err(|e| format!("parse mesh {path:?}: {e}"))
}

pub fn parse_obj(txt: &str) -> Result<TriangleMesh, String> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
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
            "f" => {
                let mut face: Vec<u32> = Vec::new();
                for tok in parts {
                    let idx_str = tok.split('/').next().unwrap_or(tok);
                    let idx: i32 = idx_str
                        .parse()
                        .map_err(|e| format!("line {}: bad index {tok}: {e}", lineno + 1))?;
                    let resolved = if idx < 0 {
                        (vertices.len() as i32 + idx) as u32
                    } else {
                        (idx - 1) as u32
                    };
                    face.push(resolved);
                }
                if face.len() < 3 {
                    return Err(format!("line {}: face needs ≥3 indices", lineno + 1));
                }
                for i in 1..face.len() - 1 {
                    indices.push([face[0], face[i], face[i + 1]]);
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
    Ok(TriangleMesh { vertices, indices })
}
