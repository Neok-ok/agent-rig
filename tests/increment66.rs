use std::{fs, path::PathBuf};

use agent_rig::{
    increment53_scene, increment65_scene, increment66_handoff_scene, increment66_scene,
    run_increment66, scene_by_id, scene_catalog, scene_catalog_v1, step_physics,
    step_physics_with_carry, vault_scene, DEFAULT_DT, INCREMENT66_STEPS,
};

fn art(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name)
}

fn body<'a>(v: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    v["bodies"]
        .as_array()
        .expect("bodies")
        .iter()
        .find(|b| b["id"] == id)
        .unwrap_or_else(|| panic!("missing {id}"))
}

fn token_at_hold(v: &serde_json::Value) {
    let walker = body(v, "walker")["position"].as_array().unwrap();
    let token = body(v, "token")["position"].as_array().unwrap();
    for (i, off) in [0.16, 0.22, 0.0].iter().enumerate() {
        let got = token[i].as_f64().unwrap();
        let want = walker[i].as_f64().unwrap() + off;
        assert!((got - want).abs() < 0.04, "token[{i}] {got} want {want}");
    }
}

#[test]
fn vault_scene_authored_from_scratch() {
    let s = vault_scene();
    assert_eq!(s.id, "vault");
    for id in ["ground", "left_wall", "right_wall", "back_wall", "plinth", "walker"] {
        assert!(s.bodies.iter().any(|b| b.id == id), "missing {id}");
    }
    for id in ["token", "npc", "bar"] {
        assert!(s.bodies.iter().all(|b| b.id != id), "vault must not author {id}");
    }
    assert!(s.pickups.is_empty());
    assert!(s.drops.is_empty());
    assert!(s.uses.is_empty());
    assert!(s.spawns.is_empty());
    assert!(s.win.is_none());
    assert!(s.transition.is_none());
    assert!(s.play_until.is_none());
    let walker = s.bodies.iter().find(|b| b.id == "walker").unwrap();
    assert!(walker.controller.is_some());
    let follow = s.camera.follow.as_ref().expect("follow camera");
    assert_eq!(follow.body, "walker");
    assert!(matches!(
        s.lights[0],
        agent_rig::Light::Directional { .. }
    ));
    let ground = s.bodies.iter().find(|b| b.id == "ground").unwrap();
    assert!(ground.material.albedo[2] > ground.material.albedo[0]);
    let plinth = s.bodies.iter().find(|b| b.id == "plinth").unwrap();
    assert!(plinth.material.emissive_intensity > 1.0);
}

#[test]
fn catalog_has_three_preserves_v1() {
    let ids: Vec<_> = scene_catalog().iter().map(|x| x.0).collect();
    assert_eq!(ids, ["courtyard", "lane", "vault"]);
    let v1: Vec<_> = scene_catalog_v1().iter().map(|x| x.0).collect();
    assert_eq!(v1, ["courtyard", "lane"]);
    assert!(scene_by_id("vault").is_some());
    let checked: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(art("artifacts/increment59/scenes.json")).unwrap())
            .unwrap();
    assert_eq!(
        checked,
        serde_json::json!([{ "id": "courtyard" }, { "id": "lane" }])
    );
}

#[test]
fn increment66_lane_identical_to_65() {
    assert_eq!(
        serde_json::to_string(&increment66_scene()).unwrap(),
        serde_json::to_string(&increment65_scene()).unwrap()
    );
    let dump = step_physics(&increment66_scene(), INCREMENT66_STEPS, DEFAULT_DT).unwrap();
    assert_eq!(dump.scene, "lane");
    assert_eq!(dump.steps, 100);
    assert_eq!(dump.used[0].at_step, 99);
    assert_eq!(dump.held[0].at_step, 66);
    assert!(dump.won.is_none() && dump.lost.is_none() && dump.dropped.is_empty());
}

#[test]
fn increment66_handoff_courtyard() {
    let handoff = increment66_handoff_scene();
    assert_eq!(handoff.transition.as_ref().unwrap().to, "vault");
    assert!(increment53_scene().transition.is_none());
    let lane = step_physics(&increment66_scene(), INCREMENT66_STEPS, DEFAULT_DT).unwrap();
    let next = step_physics_with_carry(&handoff, 1, DEFAULT_DT, Some(&lane)).unwrap();
    assert_eq!(next.scene, "courtyard");
    assert_eq!(next.steps, 1);
    assert!(next.bodies.iter().any(|b| b.id == "token"));
    assert!(next.bodies.iter().any(|b| b.id == "bar"));
    assert!(next.bodies.iter().all(|b| b.id != "npc"));
    assert_eq!(next.held[0].at_step, 66);
    let tr = next.transition.as_ref().expect("transition vault");
    assert_eq!(tr.to, "vault");
    assert_eq!(tr.at_step, 66);
    assert!(next.won.is_none() && next.lost.is_none() && next.used.is_empty());
}

#[test]
fn increment66_final_vault() {
    let lane = step_physics(&increment66_scene(), INCREMENT66_STEPS, DEFAULT_DT).unwrap();
    let next = step_physics_with_carry(
        &increment66_handoff_scene(),
        1,
        DEFAULT_DT,
        Some(&lane),
    )
    .unwrap();
    let mut final_dump =
        step_physics_with_carry(&vault_scene(), 1, DEFAULT_DT, Some(&next)).unwrap();
    agent_rig::apply_win(&mut final_dump, increment66_scene().win.as_ref());
    assert_eq!(final_dump.scene, "vault");
    assert_eq!(final_dump.steps, 1);
    assert!(final_dump.bodies.iter().any(|b| b.id == "token"));
    assert!(final_dump.bodies.iter().any(|b| b.id == "plinth"));
    assert!(final_dump.bodies.iter().all(|b| b.id != "npc" && b.id != "bar"));
    assert_eq!(final_dump.held[0].at_step, 66);
    assert!(final_dump.used.is_empty() && final_dump.lost.is_none());
    let won = final_dump.won.as_ref().expect("won");
    assert_eq!((won.kind.as_str(), won.body.as_str(), won.scene.as_str()), ("delivered", "token", "vault"));
    let v = serde_json::to_value(&final_dump).unwrap();
    token_at_hold(&v);
}

#[test]
fn sim_vault_with_and_without_carry() {
    let vault = scene_by_id("vault").expect("vault catalog");
    assert!(vault.bodies.iter().all(|b| b.id != "token"));
    let plain = step_physics(&vault, 1, DEFAULT_DT).unwrap();
    assert!(plain.bodies.iter().all(|b| b.id != "token"));
    let carry: agent_rig::PhysicsDump = serde_json::from_str(
        &fs::read_to_string(art("artifacts/increment65/physics.json")).unwrap(),
    )
    .unwrap();
    let carried = step_physics_with_carry(&vault, 1, DEFAULT_DT, Some(&carry)).unwrap();
    assert!(carried.bodies.iter().any(|b| b.id == "token"));
    assert_eq!(carried.held[0].at_step, 66);
    token_at_hold(&serde_json::to_value(&carried).unwrap());
}

#[test]
fn increment65_regression() {
    let next: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(art("artifacts/increment65/next-physics.json")).unwrap(),
    ).unwrap();
    assert_eq!(next["scene"], "courtyard");
    assert_eq!(next["won"]["kind"], "delivered");
    assert_eq!(next["won"]["scene"], "courtyard");
    assert!(!art("artifacts/increment65/final-physics.json").exists());
    assert!(!art("artifacts/increment65/final-frame.png").exists());
    assert_eq!(increment65_scene().transition.as_ref().unwrap().to, "courtyard");
}

#[test]
fn increment66_writes() {
    let out = PathBuf::from("target/test-increment66-artifacts");
    let _ = fs::remove_dir_all(&out);
    run_increment66(&out, INCREMENT66_STEPS, DEFAULT_DT, 80, 45).unwrap();
    for name in [
        "scene.json", "physics.json", "frame.png", "next-scene.json",
        "next-physics.json", "next-frame.png", "final-scene.json",
        "final-physics.json", "final-frame.png", "scenes.json",
    ] {
        let p = out.join(name);
        assert!(p.is_file(), "missing {name}");
        if name.ends_with(".png") {
            assert!(fs::metadata(&p).unwrap().len() > 256);
        }
    }
    assert_eq!(
        fs::read(out.join("physics.json")).unwrap(),
        fs::read(art("artifacts/increment65/physics.json")).unwrap()
    );
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("next-physics.json")).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    assert_eq!(next["transition"]["to"], "vault");
    assert!(next.get("won").is_none());
    let final_: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("final-physics.json")).unwrap()).unwrap();
    assert_eq!(final_["scene"], "vault");
    assert_eq!(final_["won"]["kind"], "delivered");
    let scenes: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("scenes.json")).unwrap()).unwrap();
    assert_eq!(
        scenes,
        serde_json::json!([{ "id": "courtyard" }, { "id": "lane" }, { "id": "vault" }])
    );
    let checked = art("artifacts/increment66");
    if checked.join("threejs-frame.png").is_file() {
        for name in [
            "scene.json", "physics.json", "frame.png", "threejs-frame.png",
            "next-scene.json", "next-physics.json", "next-frame.png", "next-threejs-frame.png",
            "final-scene.json", "final-physics.json", "final-frame.png", "final-threejs-frame.png",
            "scenes.json",
        ] {
            let p = checked.join(name);
            assert!(p.is_file(), "checked-in missing {name}");
            if name.ends_with(".png") {
                assert!(fs::metadata(&p).unwrap().len() > 256);
            }
        }
        assert_eq!(
            fs::read(checked.join("physics.json")).unwrap(),
            fs::read(art("artifacts/increment65/physics.json")).unwrap()
        );
        assert_eq!(
            fs::read(checked.join("next-frame.png")).unwrap(),
            fs::read(art("artifacts/increment65/next-frame.png")).unwrap()
        );
    }
}
