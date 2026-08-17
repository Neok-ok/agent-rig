use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment20_scene, increment20_scene_json, increment21_scene,
    increment21_scene_json, increment22_scene, increment22_scene_json, increment23_scene,
    increment23_scene_json, increment24_scene, increment24_scene_json, load_mesh, parse_scene,
    run_increment24, step_physics, Light, Shape, DEFAULT_DT, INCREMENT24_STEPS,
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

fn pane_volume(mat: &serde_json::Value) -> (Option<[f64; 3]>, Option<f64>, Option<f64>) {
    let Some(vol) = mat
        .get("extensions")
        .and_then(|e| e.get("KHR_materials_volume"))
    else {
        return (None, None, None);
    };
    let color = vol.get("attenuationColor").and_then(|v| v.as_array()).map(|arr| {
        [
            arr.get(0).and_then(|n| n.as_f64()).unwrap_or(1.0),
            arr.get(1).and_then(|n| n.as_f64()).unwrap_or(1.0),
            arr.get(2).and_then(|n| n.as_f64()).unwrap_or(1.0),
        ]
    });
    let dist = vol.get("attenuationDistance").and_then(|v| v.as_f64());
    let thick = vol.get("thicknessFactor").and_then(|v| v.as_f64());
    (color, dist, thick)
}

#[test]
fn increment24_does_not_mutate_prior_scene_json() {
    let s18 = parse_scene(increment18_scene_json()).expect("inc18");
    let s20 = parse_scene(increment20_scene_json()).expect("inc20");
    let s21 = parse_scene(increment21_scene_json()).expect("inc21");
    let s22 = parse_scene(increment22_scene_json()).expect("inc22");
    let s23 = parse_scene(increment23_scene_json()).expect("inc23");
    assert_eq!(s18.bodies.len(), 5, "increment 18 scene JSON must stay 5 bodies");
    assert_eq!(s20.bodies.len(), 5, "increment 20 scene JSON must stay 5 bodies");
    assert_eq!(s21.bodies.len(), 7, "increment 21 scene JSON must stay 7 bodies");
    assert_eq!(s22.bodies.len(), 7, "increment 22 scene JSON must stay 7 bodies");
    assert_eq!(s23.bodies.len(), 7, "increment 23 scene JSON must stay 7 bodies");
    for (name, json) in [
        ("18", increment18_scene_json()),
        ("20", increment20_scene_json()),
        ("21", increment21_scene_json()),
        ("22", increment22_scene_json()),
        ("23", increment23_scene_json()),
    ] {
        assert!(
            !json.contains("attenuation") && !json.contains("KHR_materials_volume"),
            "must not mutate increment {name} scene JSON with volume fields"
        );
    }
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
}

#[test]
fn pane_has_volume_attenuation_color_and_distance() {
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
        used.iter().any(|s| *s == "KHR_materials_volume"),
        "extensionsUsed must list KHR_materials_volume, got {used:?}"
    );
    let mat = &v["materials"][0];
    let (ior, transmission) = pane_ior_and_transmission(mat);
    assert!(transmission > 0.0, "keep pane transmission, got {transmission}");
    assert!(ior > 1.0, "keep pane IOR > 1, got {ior}");

    let (color, dist, thick) = pane_volume(mat);
    let color = color.expect("pane must author KHR_materials_volume.attenuationColor");
    let dist = dist.expect("pane must author KHR_materials_volume.attenuationDistance");
    let thick = thick.expect("pane must author KHR_materials_volume.thicknessFactor");
    assert!(
        (color[0] - 1.0).abs() > 0.05 || (color[1] - 1.0).abs() > 0.05 || (color[2] - 1.0).abs() > 0.05,
        "attenuationColor must not be white, got {color:?}"
    );
    let greenish = color[1] > color[0] && color[1] > color[2];
    let amberish = color[0] > 0.7 && color[1] > 0.3 && color[1] < 0.75 && color[2] < 0.35;
    assert!(
        greenish || amberish,
        "attenuationColor should be obvious green or amber, got {color:?}"
    );
    assert!(
        dist.is_finite() && dist > 0.05 && dist < 2.0,
        "attenuationDistance must be authored (not a hidden constant / +inf), got {dist}"
    );
    assert!(
        thick > 0.2,
        "thicknessFactor must be large enough for a visible path-length tint, got {thick}"
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
        "loader must read volume attenuation, distance={}",
        gm.attenuation_distance
    );
    let c = gm.attenuation_color;
    assert!(
        (c[0] - 1.0).abs() > 0.05 || (c[1] - 1.0).abs() > 0.05 || (c[2] - 1.0).abs() > 0.05,
        "loaded attenuation_color must not be white, got {c:?}"
    );
    assert!(
        gm.attenuation_distance.is_finite() && gm.attenuation_distance > 0.05,
        "loaded attenuation_distance must be authored, got {}",
        gm.attenuation_distance
    );
    assert!(
        gm.thickness > 0.2,
        "loaded thickness must be authored, got {}",
        gm.thickness
    );
}

#[test]
fn increment24_keeps_courtyard() {
    let scene = parse_scene(increment24_scene_json()).expect("increment24 JSON should parse");
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
        "keep the increment-23 courtyard including crate, bench, and pane"
    );

    let ball = body_by_id(&scene, "ball");
    assert!(
        ball.material.clearcoat > 0.5,
        "keep the clearcoat ball, got {}",
        ball.material.clearcoat
    );

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

    for b in &scene.bodies {
        let v = b.linear_velocity;
        let speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!(
            speed < 1e-6,
            "increment 24 is a still; body {} should have zero velocity, got {v:?}",
            b.id
        );
    }
}

#[test]
fn contact_involving_gltf_body_after_step() {
    let scene = increment24_scene();
    let dump = step_physics(&scene, INCREMENT24_STEPS, DEFAULT_DT).expect("physics");
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
fn increment24_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment24.sh");
    assert!(script.is_file(), "scripts/increment24.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment24-threejs.sh");
    assert!(three.is_file(), "scripts/increment24-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment24-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment24(&out, INCREMENT24_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment24");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert!(scene.bodies.len() >= 7);
    let bench = body_by_id(&scene, "bench");
    assert!(
        bench.material.sheen > 0.5,
        "written scene must keep sheen on the bench, got {}",
        bench.material.sheen
    );
    let ball = body_by_id(&scene, "ball");
    assert!(
        ball.material.clearcoat > 0.5,
        "written scene must keep clearcoat on the ball, got {}",
        ball.material.clearcoat
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
    assert!(
        col == "convex_hull" || col == "trimesh",
        "dump must record collider type for the glTF pillar, got {col}"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let hit = contacts
        .iter()
        .any(|c| c["body_a"] == "pillar" || c["body_b"] == "pillar");
    assert!(hit, "dump contacts should include the glTF pillar");

    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
