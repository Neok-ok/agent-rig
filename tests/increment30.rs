use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment20_scene, increment20_scene_json, increment21_scene,
    increment21_scene_json, increment22_scene, increment22_scene_json, increment23_scene,
    increment23_scene_json, increment24_scene, increment24_scene_json, increment25_scene,
    increment25_scene_json, increment26_scene, increment26_scene_json, increment27_scene,
    increment27_scene_json, increment28_scene, increment28_scene_json, increment29_scene,
    increment29_scene_json, increment30_scene, increment30_scene_json, parse_scene,
    run_increment30, step_physics, Joint, Light, Shape, DEFAULT_DT, INCREMENT30_STEPS,
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

fn hinge_motor(scene: &agent_rig::Scene) -> (f32, f32) {
    for j in &scene.joints {
        if let Joint::Hinge {
            body_a,
            body_b,
            motor_target_velocity,
            motor_max_force,
            ..
        } = j
        {
            if body_a == "pillar" && body_b == "lantern" {
                return (*motor_target_velocity, *motor_max_force);
            }
        }
    }
    panic!("scene missing pillar–lantern hinge");
}

fn slider_of(scene: &agent_rig::Scene) -> (&str, &str, [f32; 3], [f32; 2]) {
    for j in &scene.joints {
        if let Joint::Slider {
            body_a,
            body_b,
            axis,
            limits,
            ..
        } = j
        {
            return (body_a, body_b, *axis, *limits);
        }
    }
    panic!("scene missing slider joint");
}

#[test]
fn increment30_does_not_mutate_prior_scene_json() {
    let s18 = parse_scene(increment18_scene_json()).expect("inc18");
    let s20 = parse_scene(increment20_scene_json()).expect("inc20");
    let s21 = parse_scene(increment21_scene_json()).expect("inc21");
    let s22 = parse_scene(increment22_scene_json()).expect("inc22");
    let s23 = parse_scene(increment23_scene_json()).expect("inc23");
    let s24 = parse_scene(increment24_scene_json()).expect("inc24");
    let s25 = parse_scene(increment25_scene_json()).expect("inc25");
    let s26 = parse_scene(increment26_scene_json()).expect("inc26");
    let s27 = parse_scene(increment27_scene_json()).expect("inc27");
    let s28 = parse_scene(increment28_scene_json()).expect("inc28");
    let s29 = parse_scene(increment29_scene_json()).expect("inc29");
    assert_eq!(s18.bodies.len(), 5, "increment 18 scene JSON must stay 5 bodies");
    assert_eq!(s20.bodies.len(), 5, "increment 20 scene JSON must stay 5 bodies");
    assert_eq!(s21.bodies.len(), 7, "increment 21 scene JSON must stay 7 bodies");
    assert_eq!(s22.bodies.len(), 7, "increment 22 scene JSON must stay 7 bodies");
    assert_eq!(s23.bodies.len(), 7, "increment 23 scene JSON must stay 7 bodies");
    assert_eq!(s24.bodies.len(), 7, "increment 24 scene JSON must stay 7 bodies");
    assert_eq!(s25.bodies.len(), 7, "increment 25 scene JSON must stay 7 bodies");
    assert_eq!(s26.bodies.len(), 7, "increment 26 scene JSON must stay 7 bodies");
    assert_eq!(s27.bodies.len(), 7, "increment 27 scene JSON must stay 7 bodies");
    assert_eq!(s28.bodies.len(), 8, "increment 28 scene JSON must stay 8 bodies");
    assert_eq!(s29.bodies.len(), 9, "increment 29 scene JSON must stay 9 bodies");
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
        ("28", increment28_scene_json()),
        ("29", increment29_scene_json()),
    ] {
        assert!(
            !json.contains("motor_target_velocity"),
            "must not mutate increment {name} scene JSON with motor_target_velocity"
        );
        assert!(
            !json.contains("motor_max_force"),
            "must not mutate increment {name} scene JSON with motor_max_force"
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
    }
    match &s28.joints[0] {
        Joint::Hinge {
            motor_target_velocity,
            motor_max_force,
            ..
        } => {
            assert_eq!(
                *motor_target_velocity, 0.0,
                "increment28 hinge motor_target_velocity must stay 0"
            );
            assert_eq!(
                *motor_max_force, 0.0,
                "increment28 hinge motor_max_force must stay 0"
            );
        }
        other => panic!("increment28 joint must stay a hinge, got {other:?}"),
    }
    let (v29, f29) = hinge_motor(&s29);
    assert_eq!(v29, 0.0, "increment29 hinge motor_target_velocity must stay 0");
    assert_eq!(f29, 0.0, "increment29 hinge motor_max_force must stay 0");
    let live28 = increment28_scene();
    let (v28, f28) = hinge_motor(&live28);
    assert_eq!(v28, 0.0, "increment28_scene hinge motor must stay 0");
    assert_eq!(f28, 0.0, "increment28_scene hinge motor_max_force must stay 0");
    let live29 = increment29_scene();
    let (v29l, f29l) = hinge_motor(&live29);
    assert_eq!(v29l, 0.0, "increment29_scene hinge motor must stay 0");
    assert_eq!(f29l, 0.0, "increment29_scene hinge motor_max_force must stay 0");
    let _ = increment20_scene();
    let _ = increment21_scene();
    let _ = increment22_scene();
    let _ = increment23_scene();
    let _ = increment24_scene();
    let _ = increment25_scene();
    let _ = increment26_scene();
    let _ = increment27_scene();
}

#[test]
fn increment30_scene_hinge_has_authored_motor() {
    let parsed = parse_scene(increment30_scene_json()).expect("increment30 JSON should parse");
    let (v, f) = hinge_motor(&parsed);
    assert!(
        (v - 4.0).abs() < 1e-5,
        "increment30 scene hinge motor_target_velocity should be ~4, got {v}"
    );
    assert!(
        (f - 8.0).abs() < 1e-5,
        "increment30 scene hinge motor_max_force should be ~8, got {f}"
    );
    assert!(v.abs() > 1e-3 && f.abs() > 1e-3, "motor must be nonzero");

    let live = increment30_scene();
    let (lv, lf) = hinge_motor(&live);
    assert!(
        (lv - 4.0).abs() < 1e-5,
        "increment30_scene hinge motor_target_velocity should be ~4, got {lv}"
    );
    assert!(
        (lf - 8.0).abs() < 1e-5,
        "increment30_scene hinge motor_max_force should be ~8, got {lf}"
    );
    assert_eq!(
        live.bodies.len(),
        increment29_scene().bodies.len(),
        "must not add bodies"
    );
    assert_eq!(
        live.joints.len(),
        increment29_scene().joints.len(),
        "must not add joints"
    );
}

#[test]
fn increment30_keeps_courtyard() {
    let scene = increment30_scene();
    assert!(
        scene.bodies.len() >= 9,
        "scene must have courtyard + lantern + drawer, got {}",
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
    let has_lantern = body_by_id(&scene, "lantern").id == "lantern";
    let has_drawer = body_by_id(&scene, "drawer").id == "drawer";
    assert!(
        has_bowl
            && has_rock
            && has_ball
            && has_pillar
            && has_pane
            && has_crate
            && has_bench
            && has_lantern
            && has_drawer,
        "keep the increment-29 courtyard including drawer, lantern, crate, bench, pane"
    );

    let hinges: Vec<_> = scene
        .joints
        .iter()
        .filter(|j| matches!(j, Joint::Hinge { .. }))
        .collect();
    assert_eq!(hinges.len(), 1, "keep the lantern hinge, got {}", hinges.len());
    match hinges[0] {
        Joint::Hinge { body_a, body_b, .. } => {
            assert_eq!(body_a, "pillar");
            assert_eq!(body_b, "lantern");
        }
        _ => unreachable!(),
    }
    let (a, b, axis, limits) = slider_of(&scene);
    assert_eq!(a, "crate");
    assert_eq!(b, "drawer");
    let alen = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    assert!(alen > 0.5, "keep slider axis, got {axis:?}");
    assert!(limits[1] > limits[0], "keep slider limits");

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
    assert!((ior - 1.5).abs() < 0.05, "keep pane IOR ~1.5, got {ior}");
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
fn increment30_lantern_swung_and_dump_records_motor() {
    let scene = increment30_scene();
    let dump = step_physics(&scene, INCREMENT30_STEPS, DEFAULT_DT).expect("physics");
    assert!(
        dump.joints.len() >= 2,
        "dump must record hinge + slider, got {}",
        dump.joints.len()
    );

    let hinge = dump
        .joints
        .iter()
        .find(|j| j.kind == "hinge")
        .expect("dump missing hinge");
    assert_eq!(hinge.body_a, "pillar");
    assert_eq!(hinge.body_b, "lantern");
    let mtv = hinge
        .motor_target_velocity
        .expect("dump hinge must record motor_target_velocity");
    let mmf = hinge
        .motor_max_force
        .expect("dump hinge must record motor_max_force");
    assert!(
        (mtv - 4.0).abs() < 1e-5,
        "dump hinge motor_target_velocity should be 4, got {mtv}"
    );
    assert!(
        (mmf - 8.0).abs() < 1e-5,
        "dump hinge motor_max_force should be 8, got {mmf}"
    );

    let slider = dump
        .joints
        .iter()
        .find(|j| j.kind == "slider")
        .expect("dump missing slider");
    assert_eq!(slider.body_a, "crate");
    assert_eq!(slider.body_b, "drawer");
    let slen = (slider.axis[0] * slider.axis[0]
        + slider.axis[1] * slider.axis[1]
        + slider.axis[2] * slider.axis[2])
        .sqrt();
    assert!(slen > 0.5, "dump slider axis must be present, got {:?}", slider.axis);
    let limits = slider.limits.expect("dump slider must record limits");
    assert!(
        limits[1] > limits[0],
        "dump slider limits must be a range, got {limits:?}"
    );

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
    let com = lantern.position;
    // Increment 29 hang reference: COM ~[1.10, 0.756, 0.961], ~0.32 below
    // anchor [1.10, 1.08, 1.10], axis [1,0,0].
    let hang = [1.10_f32, 0.756, 0.961];
    let anchor = [1.10_f32, 1.08, 1.10];
    let dz = (com[2] - hang[2]).abs();
    let hung_gap = anchor[1] - com[1];
    let horiz = ((com[0] - hang[0]) * (com[0] - hang[0])
        + (com[2] - hang[2]) * (com[2] - hang[2]))
        .sqrt();
    assert!(
        dz > 0.15 || hung_gap < 0.15 || horiz > 0.15,
        "lantern COM {com:?} must be off the increment-29 hang line ~{hang:?} \
         (|z-0.961|={dz}, anchor_y-com_y={hung_gap}, horiz={horiz})"
    );
}

#[test]
fn increment30_writes_scene_dump_and_png() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment30.sh");
    assert!(script.is_file(), "scripts/increment30.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment30-threejs.sh");
    assert!(three.is_file(), "scripts/increment30-threejs.sh must exist");

    let out = PathBuf::from("target/test-increment30-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment30(&out, INCREMENT30_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment30");
    assert!(paths.scene.is_file(), "missing scene.json");
    assert!(paths.physics.is_file(), "missing physics.json");
    assert!(paths.frame.is_file(), "missing frame.png");

    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    let scene = parse_scene(&scene_txt).expect("written scene.json parses");
    assert!(scene.bodies.iter().any(|b| b.id == "drawer"));
    assert!(scene.bodies.iter().any(|b| b.id == "lantern"));
    let (v, f) = hinge_motor(&scene);
    assert!((v - 4.0).abs() < 1e-5, "written scene motor_target_velocity {v}");
    assert!((f - 8.0).abs() < 1e-5, "written scene motor_max_force {f}");
    let (a, b, _axis, limits) = slider_of(&scene);
    assert_eq!(a, "crate");
    assert_eq!(b, "drawer");
    assert!(limits[1] > limits[0]);
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
    assert!(v["bodies"].as_array().unwrap().len() >= 9);
    let joints = v["joints"].as_array().expect("dump must have joints array");
    let slider = joints
        .iter()
        .find(|j| j["kind"] == "slider")
        .expect("dump joints must record the slider");
    assert_eq!(slider["body_a"], "crate");
    assert_eq!(slider["body_b"], "drawer");
    assert!(slider["axis"].is_array());
    assert!(slider["limits"].is_array());
    let hinge = joints
        .iter()
        .find(|j| j["kind"] == "hinge")
        .expect("dump joints must record the hinge");
    assert_eq!(hinge["body_a"], "pillar");
    assert_eq!(hinge["body_b"], "lantern");
    assert!(
        hinge.get("motor_target_velocity").is_some(),
        "dump hinge must record motor_target_velocity"
    );
    assert!(
        hinge.get("motor_max_force").is_some(),
        "dump hinge must record motor_max_force"
    );
    assert!((hinge["motor_target_velocity"].as_f64().unwrap() - 4.0).abs() < 1e-5);
    assert!((hinge["motor_max_force"].as_f64().unwrap() - 8.0).abs() < 1e-5);
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
