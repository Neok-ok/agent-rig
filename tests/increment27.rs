use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment20_scene, increment20_scene_json, increment21_scene,
    increment21_scene_json, increment22_scene, increment22_scene_json, increment23_scene,
    increment23_scene_json, increment24_scene, increment24_scene_json, increment25_scene,
    increment25_scene_json, increment26_scene, increment26_scene_json, increment27_scene,
    increment27_scene_json, load_mesh, parse_scene, run_increment27, step_physics, Light, Shape,
    DEFAULT_DT, INCREMENT27_STEPS,
};

fn gltf_bodies(scene: &agent_rig::Scene) -> Vec<&agent_rig::Body> {
    scene
        .bodies
        .iter()
        .filter(|b| match &b.shape {
            Shape::Mesh { path, .. } => path.ends_with(".gltf") || path.ends_with(".glb"),
            _ => false,
        })
        .collect()
}

fn body_by_id<'a>(scene: &'a agent_rig::Scene, id: &str) -> &'a agent_rig::Body {
    scene
        .bodies
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("missing body {id}"))
}

fn pane_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pane.gltf")
}

fn pillar_gltf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("meshes/pillar.gltf")
}

fn area_light(scene: &agent_rig::Scene) -> ([f32; 3], [f32; 2], [f32; 3], f32, [f32; 3]) {
    for light in &scene.lights {
        if let Light::Area {
            position,
            size,
            color,
            intensity,
            normal,
        } = light
        {
            return (*position, *size, *color, *intensity, *normal);
        }
    }
    panic!("scene missing area light");
}

fn pane_ior_and_transmission(mat: &serde_json::Value) -> (f64, f64) {
    let mut transmission = 0.0;
    let mut ior = mat["ior"].as_f64().unwrap_or(0.0);
    if let Some(ext) = mat.get("extensions") {
        if let Some(n) = ext
            .get("KHR_materials_transmission")
            .and_then(|t| t.get("transmissionFactor"))
            .and_then(|v| v.as_f64())
        {
            transmission = n;
        }
        if let Some(n) = ext
            .get("KHR_materials_ior")
            .and_then(|t| t.get("ior"))
            .and_then(|v| v.as_f64())
        {
            ior = n;
        }
    }
    (ior, transmission)
}

fn pane_dispersion(mat: &serde_json::Value) -> Option<f64> {
    mat.get("extensions")
        .and_then(|e| e.get("KHR_materials_dispersion"))
        .and_then(|d| d.get("dispersion"))
        .and_then(|v| v.as_f64())
}

#[test]
fn increment27_does_not_mutate_prior_scene_json() {
    let s18 = parse_scene(increment18_scene_json()).expect("inc18");
    let s20 = parse_scene(increment20_scene_json()).expect("inc20");
    let s21 = parse_scene(increment21_scene_json()).expect("inc21");
    let s22 = parse_scene(increment22_scene_json()).expect("inc22");
    let s23 = parse_scene(increment23_scene_json()).expect("inc23");
    let s24 = parse_scene(increment24_scene_json()).expect("inc24");
    let s25 = parse_scene(increment25_scene_json()).expect("inc25");
    let s26 = parse_scene(increment26_scene_json()).expect("inc26");
    assert_eq!(s18.bodies.len(), 5, "increment 18 scene JSON must stay 5 bodies");
    assert_eq!(s20.bodies.len(), 5, "increment 20 scene JSON must stay 5 bodies");
    assert_eq!(s21.bodies.len(), 7, "increment 21 scene JSON must stay 7 bodies");
    assert_eq!(s22.bodies.len(), 7, "increment 22 scene JSON must stay 7 bodies");
    assert_eq!(s23.bodies.len(), 7, "increment 23 scene JSON must stay 7 bodies");
    assert_eq!(s24.bodies.len(), 7, "increment 24 scene JSON must stay 7 bodies");
    assert_eq!(s25.bodies.len(), 7, "increment 25 scene JSON must stay 7 bodies");
    assert_eq!(s26.bodies.len(), 7, "increment 26 scene JSON must stay 7 bodies");
    for (name, json) in [
        ("18", increment18_scene_json()),
        ("20", increment20_scene_json()),
        ("21", increment21_scene_json()),
        ("22", increment22_scene_json()),
        ("23", increment23_scene_json()),
        ("24", increment24_scene_json()),
        ("25", increment25_scene_json()),
        ("26", increment26_scene_json()),
    ] {
        assert!(
            !json.contains("dispersion"),
            "must not mutate increment {name} scene JSON with dispersion fields"
        );
    }
    let ball26 = body_by_id(&s26, "ball");
    assert!(
        ball26.material.iridescence > 0.5,
        "increment 26 ball iridescence must stay, got {}",
        ball26.material.iridescence
    );
    let pane26 = body_by_id(&s26, "pane");
    assert!(
        pane26.material.dispersion < 1e-6,
        "increment 26 pane must stay without scene-JSON dispersion, got {}",
        pane26.material.dispersion
    );
    let bench23 = body_by_id(&s23, "bench");
    assert!(
        bench23.material.sheen > 0.5,
        "increment 23 bench sheen must stay, got {}",
        bench23.material.sheen
    );
    let _ = increment20_scene();
    let _ = increment21_scene();
    let _ = increment22_scene();
    let _ = increment23_scene();
    let _ = increment24_scene();
    let _ = increment25_scene();
    let _ = increment26_scene();
}

#[test]
fn increment27_pane_has_authored_dispersion() {
    let path = pane_gltf_path();
    assert!(path.is_file(), "meshes/pane.gltf must be checked in");
    let txt = fs::read_to_string(&path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&txt).expect("pane.gltf is JSON");
    let used = v["extensionsUsed"]
        .as_array()
        .expect("extensionsUsed")
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>();
    assert!(
        used.iter().any(|s| *s == "KHR_materials_dispersion"),
        "extensionsUsed must list KHR_materials_dispersion, got {used:?}"
    );
    assert!(
        used.iter().any(|s| *s == "KHR_materials_volume"),
        "keep pane volume, extensionsUsed={used:?}"
    );
    assert!(
        used.iter().any(|s| *s == "KHR_materials_transmission"),
        "keep pane transmission, extensionsUsed={used:?}"
    );

    let mat = &v["materials"][0];
    let (ior, transmission) = pane_ior_and_transmission(mat);
    assert!(transmission > 0.0, "keep pane transmission, got {transmission}");
    assert!(ior > 1.0, "keep pane IOR > 1, got {ior}");
    let disp = pane_dispersion(mat).expect("pane must author KHR_materials_dispersion.dispersion");
    assert!(
        disp > 0.05,
        "authored dispersion must be > 0 (visible split), got {disp}"
    );
    assert!(
        (disp - 0.18).abs() < 1e-4,
        "authored dispersion should be 0.18, got {disp}"
    );

    let mesh = load_mesh(&path).expect("pane glTF should load");
    let gm = mesh
        .gltf_material
        .as_ref()
        .expect("loaded pane must carry glTF material");
    assert!(
        gm.has_transmission(),
        "loader must keep transmission, got {}",
        gm.transmission
    );
    assert!(
        gm.has_volume_attenuation(),
        "loader must keep volume attenuation, distance={}",
        gm.attenuation_distance
    );
    assert!(
        gm.has_dispersion(),
        "loader must read KHR_materials_dispersion, got {}",
        gm.dispersion
    );
    assert!(
        (gm.dispersion - disp as f32).abs() < 1e-5,
        "loaded dispersion must match authored {disp}, got {}",
        gm.dispersion
    );
    assert!(gm.ior > 1.0, "keep loaded IOR, got {}", gm.ior);

    let scene = parse_scene(increment27_scene_json()).expect("increment27 JSON should parse");
    let pane = body_by_id(&scene, "pane");
    assert!(
        pane.material.dispersion > 0.05,
        "scene pane must author dispersion, got {}",
        pane.material.dispersion
    );
    assert!(
        (pane.material.dispersion - gm.dispersion).abs() < 1e-5,
        "scene pane.dispersion must match glTF authored {}, got {}",
        gm.dispersion,
        pane.material.dispersion
    );

    let live = increment27_scene();
    let live_pane = body_by_id(&live, "pane");
    assert!(
        (live_pane.material.dispersion - gm.dispersion).abs() < 1e-5,
        "increment27_scene pane.dispersion must match glTF {}, got {}",
        gm.dispersion,
        live_pane.material.dispersion
    );
    let resolved = live
        .resolved_body_material(live_pane)
        .expect("resolved pane material");
    assert!(
        (resolved.dispersion - gm.dispersion).abs() < 1e-5,
        "resolved pane material.dispersion must match authored {}, got {}",
        gm.dispersion,
        resolved.dispersion
    );
}

#[test]
fn increment27_keeps_courtyard() {
    let scene = parse_scene(increment27_scene_json()).expect("increment27 JSON should parse");
    assert!(
        scene.bodies.len() >= 7,
        "scene must have >= 7 bodies, got {}",
        scene.bodies.len()
    );

    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(
        size[0] > 0.5 && size[1] > 0.4,
        "keep the authored area light size, got {size:?}"
    );
    assert!(intensity > 1.0, "area light intensity {intensity}");

    let has_dir = scene
        .lights
        .iter()
        .any(|l| matches!(l, Light::Directional { .. }));
    assert!(has_dir, "keep the directional");
    assert_eq!(scene.lights.len(), 2, "no extra lights, got {}", scene.lights.len());

    let has_bowl = scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("bowl")));
    let has_rock = scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("rock")));
    let has_ball = scene
        .bodies
        .iter()
        .any(|b| matches!(b.shape, Shape::Sphere { .. }));
    let has_pillar = body_by_id(&scene, "pillar").id == "pillar";
    let has_pane = matches!(
        &body_by_id(&scene, "pane").shape,
        Shape::Mesh { path, .. } if path.contains("pane")
    );
    let has_crate = matches!(
        &body_by_id(&scene, "crate").shape,
        Shape::Mesh { path, .. } if path.contains("crate")
    );
    let has_bench = matches!(
        &body_by_id(&scene, "bench").shape,
        Shape::Mesh { path, .. } if path.contains("bench")
    );
    assert!(
        has_bowl && has_rock && has_ball && has_pillar && has_pane && has_crate && has_bench,
        "keep the increment-26 courtyard including crate, bench, and pane"
    );

    let ball = body_by_id(&scene, "ball");
    assert!(
        ball.material.clearcoat > 0.5,
        "keep the clearcoat ball, got {}",
        ball.material.clearcoat
    );
    assert!(
        ball.material.anisotropy > 0.5,
        "keep the anisotropy ball, got {}",
        ball.material.anisotropy
    );
    assert!(
        ball.material.iridescence > 0.5,
        "keep the iridescence ball, got {}",
        ball.material.iridescence
    );
    assert_eq!(ball.position, [-1.10, 0.36, 0.10]);

    let bench = body_by_id(&scene, "bench");
    assert!(
        bench.material.sheen > 0.5,
        "keep the sheen bench, got {}",
        bench.material.sheen
    );
    let c = bench.material.sheen_color;
    assert!(
        c[0] > 0.5 && c[1] < 0.35 && c[2] < 0.45,
        "keep bench sheen_color, got {c:?}"
    );

    let cam = &scene.camera;
    assert_eq!(cam.position, [3.6, 2.35, 5.2]);
    assert_eq!(cam.look_at, [0.1, 0.38, 0.0]);

    let gltfs = gltf_bodies(&scene);
    assert!(
        gltfs.len() >= 2,
        "expect pillar + pane glTF bodies, got {}",
        gltfs.len()
    );

    let pillar_txt = fs::read_to_string(pillar_gltf_path()).unwrap();
    let pv: serde_json::Value = serde_json::from_str(&pillar_txt).unwrap();
    assert!(
        pv["materials"][0].get("occlusionTexture").is_some(),
        "must keep pillar occlusionTexture (AO)"
    );

    let pane_txt = fs::read_to_string(pane_gltf_path()).unwrap();
    let pane_v: serde_json::Value = serde_json::from_str(&pane_txt).unwrap();
    let (ior, transmission) = pane_ior_and_transmission(&pane_v["materials"][0]);
    assert!(transmission > 0.0, "keep pane transmission, got {transmission}");
    assert!(ior > 1.0, "keep pane IOR > 1, got {ior}");
    assert!(
        pane_v["extensionsUsed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x.as_str() == Some("KHR_materials_volume")),
        "keep pane volume attenuation"
    );

    let live = increment27_scene();
    assert_eq!(live.camera.position, scene.camera.position);
    assert_eq!(live.camera.look_at, scene.camera.look_at);
    assert_eq!(live.bodies.len(), scene.bodies.len());
    assert_eq!(live.lights.len(), scene.lights.len());

    for b in &scene.bodies {
        let v = b.linear_velocity;
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!(
            speed < 1e-6,
            "increment 27 is a still; body {} should have zero velocity, got {v:?}",
            b.id
        );
    }
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment27_scene();
    let dump = step_physics(&scene, INCREMENT27_STEPS, DEFAULT_DT).expect("physics");
    let hit = dump
        .contacts
        .iter()
        .any(|c| c.body_a == "pillar" || c.body_b == "pillar");
    assert!(
        hit,
        "expected a contact involving the glTF pillar, contacts={:?}",
        dump.contacts
    );
    let body = dump
        .bodies
        .iter()
        .find(|b| b.id == "pillar")
        .expect("dump missing pillar");
    assert!(
        body.collider == "convex_hull" || body.collider == "trimesh",
        "glTF body collider should be convex_hull or trimesh, got {}",
        body.collider
    );
    assert_eq!(body.collider, "convex_hull", "pillar stays convex_hull");

    let grounded = dump.contacts.iter().any(|c| {
        (c.body_a == "pillar" || c.body_b == "pillar")
            && (c.body_a == "ground" || c.body_b == "ground")
    });
    assert!(
        grounded,
        "expected a ground–pillar contact, contacts={:?}",
        dump.contacts
    );
}

#[test]
fn increment27_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment27.sh");
    assert!(script.is_file(), "scripts/increment27.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment27-threejs.sh");
    assert!(three.is_file(), "scripts/increment27-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment27-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment27(&out, INCREMENT27_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment27");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert!(scene.bodies.len() >= 7);
    let pane = body_by_id(&scene, "pane");
    assert!(
        pane.material.dispersion > 0.05,
        "written scene must author dispersion on the pane, got {}",
        pane.material.dispersion
    );
    let ball = body_by_id(&scene, "ball");
    assert!(
        ball.material.iridescence > 0.5,
        "written scene must keep iridescence on the ball, got {}",
        ball.material.iridescence
    );
    assert!(
        ball.material.anisotropy > 0.5,
        "written scene must keep anisotropy on the ball, got {}",
        ball.material.anisotropy
    );
    assert!(
        ball.material.clearcoat > 0.5,
        "written scene must keep clearcoat on the ball, got {}",
        ball.material.clearcoat
    );
    let bench = body_by_id(&scene, "bench");
    assert!(
        bench.material.sheen > 0.5,
        "written scene must keep sheen on the bench, got {}",
        bench.material.sheen
    );
    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(
        size[0] > 0.5 && size[1] > 0.4,
        "written scene must author area size, got {size:?}"
    );
    assert!(intensity > 0.0);
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pane")))
    );
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pillar")))
    );
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("crate")))
    );
    assert!(
        scene
            .bodies
            .iter()
            .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("bench")))
    );

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 7);
    assert!(v["contacts"].is_array());
    let pillar_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "pillar")
        .expect("dump should record the pillar");
    let col = pillar_state["collider"].as_str().unwrap_or("");
    assert_eq!(col, "convex_hull", "dump must record convex_hull for the pillar, got {col}");
    let contacts = v["contacts"].as_array().unwrap();
    let hit = contacts
        .iter()
        .any(|c| c["body_a"] == "pillar" || c["body_b"] == "pillar");
    assert!(hit, "dump contacts should include the glTF pillar");
    let grounded = contacts.iter().any(|c| {
        let a = c["body_a"].as_str().unwrap_or("");
        let b = c["body_b"].as_str().unwrap_or("");
        (a == "pillar" || b == "pillar") && (a == "ground" || b == "ground")
    });
    assert!(grounded, "dump contacts should include ground–pillar");

    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
