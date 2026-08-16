//! Tiny mesh loader: OBJ (v / vt / f) and a constrained glTF/GLB triangle mesh.

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
    Ok(TriangleMesh {
        vertices,
        indices,
        texcoords,
        tex_indices,
    })
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
