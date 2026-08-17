use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment20_scene, increment20_scene_json, increment21_scene,
    increment21_scene_json, increment22_scene, increment22_scene_json, increment23_scene,
    increment23_scene_json, increment24_scene, increment24_scene_json, increment25_scene,
    increment25_scene_json, increment26_scene, increment26_scene_json, increment27_scene,
    increment27_scene_json, increment28_scene, increment28_scene_json, parse_scene,
    run_increment28, step_physics, Joint, Light, Shape, DEFAULT_DT, INCREMENT28_STEPS,
};

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

#[test]
fn increment28_does_not_mutate_prior_scene_json() {
    let s18 = parse_scene(increment18_scene_json()).expect("inc18");
    let s20 = parse_scene(increment20_scene_json()).expect("inc20");
    let s21 = parse_scene(increment21_scene_json()).expect("inc21");
    let s22 = parse_scene(increment22_scene_json()).expect("inc22");
    let s23 = parse_scene(increment23_scene_json()).expect("inc23");
    let s24 = parse_scene(increment24_scene_json()).expect("inc24");
    let s25 = parse_scene(increment25_scene_json()).expect("inc25");
    let s26 = parse_scene(increment26_scene_json()).expect("inc26");
    let s27 = parse_scene(increment27_scene_json()).expect("inc27");
    assert_eq!(s18.bodies.len(), 5, "increment 18 scene JSON must stay 5 bodies");
    assert_eq!(s20.bodies.len(), 5, "increment 20 scene JSON must stay 5 bodies");
    assert_eq!(s21.bodies.len(), 7, "increment 21 scene JSON must stay 7 bodies");
    assert_eq!(s22.bodies.len(), 7, "increment 22 scene JSON must stay 7 bodies");
    assert_eq!(s23.bodies.len(), 7, "increment 23 scene JSON must stay 7 bodies");
    assert_eq!(s24.bodies.len(), 7, "increment 24 scene JSON must stay 7 bodies");
    assert_eq!(s25.bodies.len(), 7, "increment 25 scene JSON must stay 7 bodies");
    assert_eq!(s26.bodies.len(), 7, "increment 26 scene JSON must stay 7 bodies");
    assert_eq!(s27.bodies.len(), 7, "increment 27 scene JSON must stay 7 bodies");
    for (name, json) in [
        ("18", increment18_scene_json()),
        ("20", increment20_scene_json()),
        ("21", increment21_scene_json()),
        ("22", increment22_scene_json()),
        ("23", increment23_scene_json()),
        ("24", increment24_scene_json()),
        ("25", increment25_scene_json()),
        ("26", increment26_scene_json()),
        ("27", increment27_scene_json()),
    ] {
        assert!(
            !json.contains("lantern"),
            "must not mutate increment {name} scene JSON with a lantern"
        );
        assert!(
            !json.contains("\"joints\""),
            "must not mutate increment {name} scene JSON with joints"
        );
    }
    for (name, scene) in [
        ("18", s18),
        ("20", s20),
        ("21", s21),
        ("22", s22),
        ("23", s23),
        ("24", s24),
        ("25", s25),
        ("26", s26),
        ("27", s27),
    ] {
        assert!(
            scene.joints.is_empty(),
            "increment {name} joints must stay empty, got {}",
            scene.joints.len()
        );
        assert!(
            scene.bodies.iter().all(|b| b.id != "lantern"),
            "increment {name} must not have a lantern body"
        );
    }
    let live27 = increment27_scene();
    assert!(live27.joints.is_empty(), "increment27_scene joints must stay empty");
    assert!(
        live27.bodies.iter().all(|b| b.id != "lantern"),
        "increment27_scene must not grow a lantern"
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
fn increment28_scene_has_lantern_and_hinge() {
    let parsed = parse_scene(increment28_scene_json()).expect("increment28 JSON should parse");
    let lantern = body_by_id(&parsed, "lantern");
    match lantern.shape {
        Shape::Sphere { radius } => {
            assert!(
                (0.10..=0.14).contains(&radius),
                "lantern sphere radius should be ~0.10–0.14, got {radius}"
            );
        }
        _ => panic!("lantern should be a sphere, got {:?}", lantern.shape),
    }
    assert!(lantern.mass > 0.0, "lantern must be dynamic, mass={}", lantern.mass);

    assert_eq!(parsed.joints.len(), 1, "expect one authored hinge, got {}", parsed.joints.len());
    match &parsed.joints[0] {
        Joint::Hinge {
            body_a,
            body_b,
            anchor,
            axis,
        } => {
            assert_eq!(body_a, "pillar");
            assert_eq!(body_b, "lantern");
            assert!(
                anchor.iter().any(|c| c.abs() > 1e-4),
                "hinge anchor must be present, got {anchor:?}"
            );
            let alen = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
            assert!(alen > 0.5, "hinge axis must be present, got {axis:?}");
            assert!(
                axis[1].abs() < 0.5,
                "hinge axis should be horizontal (not Y) so gravity hangs, got {axis:?}"
            );
        }
    }

    let live = increment28_scene();
    assert!(
        live.bodies.iter().any(|b| b.id == "lantern"),
        "increment28_scene must include the lantern"
    );
    assert_eq!(live.joints.len(), 1);
    match &live.joints[0] {
        Joint::Hinge { body_a, body_b, .. } => {
            assert_eq!(body_a, "pillar");
            assert_eq!(body_b, "lantern");
        }
    }
}

#[test]
fn increment28_keeps_courtyard() {
    let scene = increment28_scene();
    assert!(
        scene.bodies.len() >= 8,
        "scene must have courtyard + lantern, got {}",
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
        .any(|b| b.id == "ball" && matches!(b.shape, Shape::Sphere { .. }));
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
        "keep the increment-27 courtyard including crate, bench, and pane"
    );

    let ball = body_by_id(&scene, "ball");
    assert!(
        ball.material.clearcoat > 0.5,
        "keep the clearcoat ball, got {}",
        ball.material.clearcoat
    );
    assert!(
        (ball.material.clearcoat_roughness - 0.08).abs() < 1e-5,
        "keep clearcoat_roughness 0.08, got {}",
        ball.material.clearcoat_roughness
    );
    assert!(
        ball.material.anisotropy > 0.5,
        "keep the anisotropy ball, got {}",
        ball.material.anisotropy
    );
    assert!(
        (ball.material.anisotropy_rotation - 0.6).abs() < 1e-5,
        "keep anisotropy_rotation 0.6, got {}",
        ball.material.anisotropy_rotation
    );
    assert!(
        ball.material.iridescence > 0.5,
        "keep the iridescence ball, got {}",
        ball.material.iridescence
    );
    assert!(
        (ball.material.iridescence_ior - 1.3).abs() < 1e-5,
        "keep iridescence_ior 1.3, got {}",
        ball.material.iridescence_ior
    );
    assert!(
        (ball.material.iridescence_thickness - 380.0).abs() < 1e-3,
        "keep iridescence_thickness 380, got {}",
        ball.material.iridescence_thickness
    );
    assert_eq!(ball.position, [-1.10, 0.36, 0.10]);

    let bench = body_by_id(&scene, "bench");
    assert!(
        bench.material.sheen > 0.5,
        "keep the sheen bench, got {}",
        bench.material.sheen
    );
    assert!(
        (bench.material.sheen_roughness - 0.4).abs() < 1e-5,
        "keep sheen_roughness 0.4, got {}",
        bench.material.sheen_roughness
    );
    let c = bench.material.sheen_color;
    assert!(
        c[0] > 0.5 && c[1] < 0.35 && c[2] < 0.45,
        "keep bench sheen_color, got {c:?}"
    );

    let pane = body_by_id(&scene, "pane");
    assert!(
        pane.material.dispersion > 0.05,
        "keep pane dispersion, got {}",
        pane.material.dispersion
    );

    let cam = &scene.camera;
    assert_eq!(cam.position, [3.6, 2.35, 5.2]);
    assert_eq!(cam.look_at, [0.1, 0.38, 0.0]);

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
        (ior - 1.5).abs() < 0.05,
        "keep pane IOR ~1.5, got {ior}"
    );
    let used = pane_v["extensionsUsed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x.as_str())
        .collect::<Vec<_>>();
    assert!(
        used.iter().any(|s| *s == "KHR_materials_volume"),
        "keep pane volume attenuation"
    );
    assert!(
        used.iter().any(|s| *s == "KHR_materials_dispersion"),
        "keep pane dispersion"
    );
    let vol = &pane_v["materials"][0]["extensions"]["KHR_materials_volume"];
    let att = vol["attenuationColor"]
        .as_array()
        .expect("attenuationColor");
    let ac = [
        att[0].as_f64().unwrap(),
        att[1].as_f64().unwrap(),
        att[2].as_f64().unwrap(),
    ];
    assert!(
        (ac[0] - 0.20).abs() < 0.02 && (ac[1] - 0.85).abs() < 0.02 && (ac[2] - 0.28).abs() < 0.02,
        "keep volume attenuationColor [0.20, 0.85, 0.28], got {ac:?}"
    );
    let ad = vol["attenuationDistance"].as_f64().unwrap();
    assert!(
        (ad - 0.45).abs() < 0.02,
        "keep attenuationDistance 0.45, got {ad}"
    );
}

#[test]
fn increment28_lantern_hangs_and_dump_records_joint() {
    let scene = increment28_scene();
    let lantern0 = body_by_id(&scene, "lantern");
    let start_y = lantern0.position[1];
    let Joint::Hinge { anchor, .. } = &scene.joints[0];
    let anchor_y = anchor[1];

    let dump = step_physics(&scene, INCREMENT28_STEPS, DEFAULT_DT).expect("physics");
    assert_eq!(dump.joints.len(), 1, "dump must record the hinge, got {}", dump.joints.len());
    let j = &dump.joints[0];
    assert_eq!(j.kind, "hinge");
    assert_eq!(j.body_a, "pillar");
    assert_eq!(j.body_b, "lantern");
    assert!(
        j.anchor.iter().any(|c| c.abs() > 1e-4),
        "dump joint anchor must be present, got {:?}",
        j.anchor
    );
    let alen = (j.axis[0] * j.axis[0] + j.axis[1] * j.axis[1] + j.axis[2] * j.axis[2]).sqrt();
    assert!(alen > 0.5, "dump joint axis must be present, got {:?}", j.axis);

    let pillar = dump
        .bodies
        .iter()
        .find(|b| b.id == "pillar")
        .expect("dump missing pillar");
    assert_eq!(pillar.collider, "convex_hull", "pillar stays convex_hull");

    let grounded = dump.contacts.iter().any(|c| {
        (c.body_a == "pillar" || c.body_b == "pillar")
            && (c.body_a == "ground" || c.body_b == "ground")
    });
    assert!(
        grounded,
        "expected a ground–pillar contact, contacts={:?}",
        dump.contacts
    );

    let lantern = dump
        .bodies
        .iter()
        .find(|b| b.id == "lantern")
        .expect("dump missing lantern");
    assert_eq!(lantern.collider, "ball", "lantern collider should be ball");
    let com_y = lantern.position[1];
    assert!(
        com_y < start_y - 0.08,
        "lantern COM should leave the authored T-pose start y={start_y}, got y={com_y}"
    );
    // Primary hang check: sphere COM must sit clearly below the world-space hinge.
    assert!(
        com_y < anchor_y - 0.15,
        "lantern COM y={com_y} must hang clearly below hinge anchor y={anchor_y} (not a T-pose)"
    );
}

#[test]
fn increment28_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment28.sh");
    assert!(script.is_file(), "scripts/increment28.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment28-threejs.sh");
    assert!(three.is_file(), "scripts/increment28-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment28-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment28(&out, INCREMENT28_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment28");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert!(scene.bodies.iter().any(|b| b.id == "lantern"));
    assert_eq!(scene.joints.len(), 1);
    match &scene.joints[0] {
        Joint::Hinge { body_a, body_b, .. } => {
            assert_eq!(body_a, "pillar");
            assert_eq!(body_b, "lantern");
        }
    }
    let pane = body_by_id(&scene, "pane");
    assert!(
        pane.material.dispersion > 0.05,
        "written scene must keep dispersion on the pane, got {}",
        pane.material.dispersion
    );
    let ball = body_by_id(&scene, "ball");
    assert!(ball.material.iridescence > 0.5);
    assert!(ball.material.anisotropy > 0.5);
    assert!(ball.material.clearcoat > 0.5);
    let bench = body_by_id(&scene, "bench");
    assert!(bench.material.sheen > 0.5);
    let (_pos, size, _color, intensity, _n) = area_light(&scene);
    assert!(size[0] > 0.5 && size[1] > 0.4);
    assert!(intensity > 0.0);
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pane"))));
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("pillar"))));
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("crate"))));
    assert!(scene
        .bodies
        .iter()
        .any(|b| matches!(&b.shape, Shape::Mesh { path, .. } if path.contains("bench"))));

    let phys_txt = fs::read_to_string(&paths.physics).unwrap();
    let v: serde_json::Value = serde_json::from_str(&phys_txt).unwrap();
    assert!(v["bodies"].as_array().unwrap().len() >= 8);
    let joints = v["joints"].as_array().expect("dump must have joints array");
    assert!(!joints.is_empty(), "dump joints must record the hinge");
    assert_eq!(joints[0]["kind"], "hinge");
    assert_eq!(joints[0]["body_a"], "pillar");
    assert_eq!(joints[0]["body_b"], "lantern");
    assert!(joints[0]["anchor"].is_array());
    assert!(joints[0]["axis"].is_array());
    let pillar_state = v["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == "pillar")
        .expect("dump should record the pillar");
    assert_eq!(
        pillar_state["collider"].as_str().unwrap_or(""),
        "convex_hull"
    );
    let contacts = v["contacts"].as_array().unwrap();
    let grounded = contacts.iter().any(|c| {
        let a = c["body_a"].as_str().unwrap_or("");
        let b = c["body_b"].as_str().unwrap_or("");
        (a == "pillar" || b == "pillar") && (a == "ground" || b == "ground")
    });
    assert!(grounded, "dump contacts should include ground–pillar");

    let png_len = fs::metadata(&paths.frame).unwrap().len();
    assert!(png_len > 256, "png too small: {png_len}");
}
