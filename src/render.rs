//! Tiny CPU Cook-Torrance raytracer with procedural IBL (spheres and oriented boxes).

use image::{Rgb, RgbImage};
use std::path::Path;

use crate::scene::{Light, Scene, Shape};

pub const FRAME_WIDTH: u32 = 800;
pub const FRAME_HEIGHT: u32 = 450;

const AA: u32 = 2;
const EPS: f32 = 1e-3;
const PI: f32 = std::f32::consts::PI;
const EXPOSURE: f32 = 0.82;
const SH_SAMPLES: u32 = 96;

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
}

enum PrimKind {
    Sphere { radius: f32 },
    Box { half: V3 },
}

struct Hit {
    t: f32,
    p: V3,
    n: V3,
    albedo: V3,
    roughness: f32,
    metallic: f32,
}

/// Order-2 SH of the procedural HDR environment (y-up).
struct EnvSh {
    c: [V3; 9],
}

pub fn render_scene(scene: &Scene, width: u32, height: u32) -> RgbImage {
    let prims: Vec<Prim> = scene
        .bodies
        .iter()
        .map(|b| {
            let kind = match b.shape {
                Shape::Sphere { radius } => PrimKind::Sphere { radius },
                Shape::Box { size } => PrimKind::Box {
                    half: V3::new(size[0] * 0.5, size[1] * 0.5, size[2] * 0.5),
                },
            };
            Prim {
                kind,
                center: V3::from_arr(b.position),
                rotation: Quat::from_wxyz(b.rotation_wxyz),
                albedo: V3::from_arr(b.material.albedo),
                roughness: b.material.roughness.clamp(0.04, 1.0),
                metallic: b.material.metallic.clamp(0.0, 1.0),
            }
        })
        .collect();

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
    match closest_hit(orig, dir, prims, 0.0, f32::MAX) {
        None => env_radiance(dir),
        Some(h) => shade(&h, dir, prims, lights, env),
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
    match p.kind {
        PrimKind::Sphere { radius } => intersect_sphere(orig, dir, p, radius, tmin, tmax),
        PrimKind::Box { half } => intersect_obb(orig, dir, p, half, tmin, tmax),
    }
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
    Hit {
        t,
        p: pnt,
        n,
        albedo: prim.albedo,
        roughness: prim.roughness,
        metallic: prim.metallic,
    }
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

    let ao = contact_ao(h.p, n, prims);

    // Diffuse irradiance from the procedural sky (SH, cosine-convolved, already E/π).
    let irradiance = sh_irradiance(env, n);
    let f_diff = fresnel_schlick(n_dot_v, f0);
    let k_d = V3::new(1.0 - f_diff.x, 1.0 - f_diff.y, 1.0 - f_diff.z).mul(1.0 - h.metallic);
    let diffuse_ibl = k_d.hadamard(h.albedo).hadamard(irradiance).mul(ao);

    // Specular IBL: roughness-blurred environment in the reflection direction.
    let r = n.mul(2.0 * n_dot_v).sub(v).norm();
    let spec_env = env_specular(r, n, h.roughness);
    let spec_brdf = env_brdf(n_dot_v, h.roughness, f0);
    let spec_ao = 0.25 + 0.75 * ao;
    let specular_ibl = spec_env.hadamard(spec_brdf).mul(spec_ao);

    let mut color = diffuse_ibl.add(specular_ibl);

    for light in lights {
        let Light::Directional {
            direction,
            color: lcol,
            intensity,
        } = light;
        let ldir = V3::from_arr(*direction).norm();
        let l = ldir.mul(-1.0);
        let n_dot_l = n.dot(l).max(0.0);
        if n_dot_l <= 0.0 {
            continue;
        }
        let shadow_orig = h.p.add(n.mul(EPS * 4.0));
        if closest_hit(shadow_orig, l, prims, EPS, f32::MAX).is_some() {
            continue;
        }
        let radiance = V3::from_arr(*lcol).mul(*intensity);
        color = color.add(cook_torrance(
            h.albedo, h.roughness, h.metallic, n, v, l, n_dot_l, radiance,
        ));
    }
    color
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
    match closest_hit(orig, dir, prims, EPS, max_dist) {
        None => 1.0,
        Some(h) => {
            let t = (h.t / max_dist).clamp(0.0, 1.0);
            0.10 + 0.90 * t.sqrt()
        }
    }
}

/// HDR gradient sky + tight horizon band + soft sun aureole (linear radiance).
fn env_radiance(dir: V3) -> V3 {
    let d = dir.norm();
    let y = d.y;
    let ground = V3::new(0.050, 0.046, 0.042);
    let horizon = V3::new(0.58, 0.66, 0.80);
    let zenith = V3::new(0.10, 0.20, 0.48);

    let base = if y <= 0.0 {
        let t = (-y).clamp(0.0, 1.0).powf(0.35);
        lerp(horizon, ground, t)
    } else {
        // Blue takes over quickly above the horizon so the frame reads as sky, not white.
        let t = y.clamp(0.0, 1.0).powf(0.45);
        lerp(horizon, zenith, t)
    };

    // Thin bright horizon (glossy reflections + backdrop), not a white hemisphere.
    let hz = (-y * y * 18.0).exp();
    let glow = V3::new(0.95, 0.88, 0.78).mul(0.28 * hz);

    let sun = V3::new(0.45, 1.0, 0.35).norm();
    let m = d.dot(sun).max(0.0);
    let aureole = m.powf(80.0) * 1.6 + m.powf(16.0) * 0.12;
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
    V3::new(e.x.max(0.0), e.y.max(0.0), e.z.max(0.0)).mul(0.85)
}

/// Roughness-blurred environment sample (cone opens toward N; blends to horizon/zenith).
fn env_specular(r: V3, n: V3, roughness: f32) -> V3 {
    let a = (roughness * roughness).clamp(0.0, 1.0);
    let dir = r.mul((1.0 - a).max(1e-4)).add(n.mul(a)).norm();
    let sharp = env_radiance(dir);
    let horiz = {
        let h = V3::new(dir.x, 0.0, dir.z);
        if h.len() < 1e-6 {
            env_radiance(V3::new(1.0, 0.0, 0.0))
        } else {
            env_radiance(h.norm())
        }
    };
    let zenith = env_radiance(V3::new(0.0, 1.0, 0.0));
    let mip = lerp(horiz, zenith, 0.35);
    lerp(sharp, mip, a)
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
