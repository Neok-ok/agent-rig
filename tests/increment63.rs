use std::fs;
use std::path::PathBuf;

use agent_rig::{
    increment18_scene_json, increment19_scene_json, increment20_scene_json,
    increment21_scene_json, increment22_scene_json, increment23_scene_json,
    increment24_scene_json, increment25_scene_json, increment26_scene_json,
    increment27_scene_json, increment28_scene_json, increment29_scene_json,
    increment30_scene_json, increment31_scene_json, increment32_scene_json,
    increment33_scene_json, increment34_scene_json, increment35_scene_json,
    increment36_scene_json, increment37_scene_json, increment38_scene_json,
    increment39_scene_json, increment40_scene_json, increment41_scene_json,
    increment42_scene_json, increment43_scene_json, increment44_scene_json,
    increment45_scene_json, increment46_scene_json, increment47_scene_json,
    increment48_scene_json, increment49_scene_json, increment50_scene_json,
    increment51_scene_json, increment52_scene_json, increment53_scene_json,
    increment54_scene_json, increment55_scene_json, increment56_scene_json,
    increment57_scene_json, increment58_scene_json, increment59_scene_json,
    increment60_scene_json, increment61_scene_json, increment62_scene,
    increment62_scene_json, increment63_scene, increment63_scene_json,
    parse_scene, run_increment63, step_catalog_scene_with_carry, step_physics,
    DEFAULT_DT, INCREMENT62_STEPS, INCREMENT63_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 45] {
    [
        ("18", increment18_scene_json()),
        ("19", increment19_scene_json()),
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
        ("30", increment30_scene_json()),
        ("31", increment31_scene_json()),
        ("32", increment32_scene_json()),
        ("33", increment33_scene_json()),
        ("34", increment34_scene_json()),
        ("35", increment35_scene_json()),
        ("36", increment36_scene_json()),
        ("37", increment37_scene_json()),
        ("38", increment38_scene_json()),
        ("39", increment39_scene_json()),
        ("40", increment40_scene_json()),
        ("41", increment41_scene_json()),
        ("42", increment42_scene_json()),
        ("43", increment43_scene_json()),
        ("44", increment44_scene_json()),
        ("45", increment45_scene_json()),
        ("46", increment46_scene_json()),
        ("47", increment47_scene_json()),
        ("48", increment48_scene_json()),
        ("49", increment49_scene_json()),
        ("50", increment50_scene_json()),
        ("51", increment51_scene_json()),
        ("52", increment52_scene_json()),
        ("53", increment53_scene_json()),
        ("54", increment54_scene_json()),
        ("55", increment55_scene_json()),
        ("56", increment56_scene_json()),
        ("57", increment57_scene_json()),
        ("58", increment58_scene_json()),
        ("59", increment59_scene_json()),
        ("60", increment60_scene_json()),
        ("61", increment61_scene_json()),
        ("62", increment62_scene_json()),
    ]
}

fn has_win_key(json: &str) -> bool {
    let v: serde_json::Value = serde_json::from_str(json).expect("parse scene json");
    v.get("win").is_some()
}

#[test]
fn increment63_does_not_mutate_prior() {
    assert!(
        increment62_scene().win.is_none(),
        "increment62_scene must stay won-free"
    );
    assert!(
        increment62_scene().drops.is_empty(),
        "increment62 still no drops"
    );
    let tr = increment62_scene()
        .transition
        .expect("increment62 still has transition");
    assert_eq!(tr.to, "courtyard");
    for (name, json) in prior_scene_jsons() {
        let parsed = parse_scene(json).unwrap_or_else(|e| panic!("increment {name} parse: {e}"));
        assert!(
            parsed.win.is_none(),
            "increment {name} scene.win must stay none"
        );
        assert!(
            !has_win_key(json),
            "increment {name} JSON must stay without a win key"
        );
    }
    let inc63 = increment63_scene_json();
    assert!(has_win_key(inc63), "increment63 json HAS win");
    let v: serde_json::Value = serde_json::from_str(inc63).expect("inc63 json");
    assert_eq!(v["win"]["kind"], "delivered");
    assert_eq!(v["win"]["body"], "token");
}

#[test]
fn increment63_scene_is_lane_plus_win() {
    let scene = increment63_scene();
    assert_eq!(scene.id, "lane");
    let win = scene.win.as_ref().expect("win");
    assert_eq!(win.kind, "delivered");
    assert_eq!(win.body, "token");
    assert!(scene.drops.is_empty(), "drops must be empty");
    let tr = scene.transition.as_ref().expect("transition");
    assert_eq!(tr.to, "courtyard");
    assert!(scene.pickups.iter().any(|p| p.hold), "has hold pickup");
    assert!(scene.bodies.iter().any(|b| b.id == "npc"), "has npc");
}

#[test]
fn increment62_next_still_won_free() {
    let art = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts/increment62/next-physics.json");
    if art.is_file() {
        let next: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&art).unwrap()).unwrap();
        assert!(
            next.get("won").is_none(),
            "increment62 next-physics must have no won key"
        );
    }
    let lane = step_physics(&increment62_scene(), INCREMENT62_STEPS, DEFAULT_DT)
        .expect("increment62 lane");
    let next = step_catalog_scene_with_carry(
        "courtyard",
        INCREMENT62_STEPS,
        DEFAULT_DT,
        Some(&lane),
    )
    .expect("carry increment62 into courtyard");
    assert!(next.won.is_none(), "increment62 courtyard carry dump.won is none");
    let v = serde_json::to_value(&next).expect("serialize increment62 next");
    assert!(v.get("won").is_none(), "increment62 next dump omits won");
}

#[test]
fn increment63_lane_dump() {
    let dump = step_physics(&increment63_scene(), INCREMENT63_STEPS, DEFAULT_DT)
        .expect("increment63 physics");
    assert_eq!(dump.scene, "lane");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps 85..=115, got {}",
        dump.steps
    );
    let held = dump
        .held
        .iter()
        .find(|h| h.id == "token")
        .expect("dump.held token");
    assert_eq!(held.by, "walker");
    assert_eq!(held.at_step, 66);
    let tr = dump.transition.as_ref().expect("dump.transition");
    assert_eq!(tr.to, "courtyard");
    let walker = dump
        .bodies
        .iter()
        .find(|b| b.id == "walker")
        .expect("dump walker");
    let token = dump
        .bodies
        .iter()
        .find(|b| b.id == "token")
        .expect("dump token");
    let expect = [
        walker.position[0] + 0.16,
        walker.position[1] + 0.22,
        walker.position[2] + 0.00,
    ];
    for i in 0..3 {
        assert!(
            (token.position[i] - expect[i]).abs() < 0.04,
            "token at hold_offset[{i}] got {} want {}",
            token.position[i],
            expect[i]
        );
    }
    assert!(dump.won.is_none(), "lane dump.won omitted");
    let v = serde_json::to_value(&dump).expect("serialize dump");
    assert!(v.get("won").is_none(), "serialized lane dump must omit won");
}

#[test]
fn increment63_next_won() {
    let out = PathBuf::from("target/test-increment63-next");
    let _ = fs::remove_dir_all(&out);
    let _paths = run_increment63(&out, INCREMENT63_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment63");
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    let bodies = next["bodies"].as_array().expect("next bodies");
    assert!(bodies.iter().any(|b| b["id"] == "token"), "HAS token");
    assert!(bodies.iter().any(|b| b["id"] == "bar"), "HAS bar");
    assert!(bodies.iter().all(|b| b["id"] != "npc"), "NO npc");
    let walker = bodies.iter().find(|b| b["id"] == "walker").expect("walker");
    let token = bodies.iter().find(|b| b["id"] == "token").expect("token");
    let wp = walker["position"].as_array().expect("walker pos");
    let tp = token["position"].as_array().expect("token pos");
    let expect = [
        wp[0].as_f64().unwrap() + 0.16,
        wp[1].as_f64().unwrap() + 0.22,
        wp[2].as_f64().unwrap() + 0.00,
    ];
    for i in 0..3 {
        let got = tp[i].as_f64().unwrap();
        assert!(
            (got - expect[i]).abs() < 0.08,
            "token[{i}] got {got} want {}",
            expect[i]
        );
    }
    let won = next.get("won").expect("dump.won");
    assert_eq!(won["kind"], "delivered");
    assert_eq!(won["body"], "token");
    assert_eq!(won["scene"], "courtyard");
}

#[test]
fn increment63_writes() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment63.sh");
    assert!(script.is_file(), "scripts/increment63.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment63-threejs.sh");
    assert!(three.is_file(), "scripts/increment63-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment63-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment63(&out, INCREMENT63_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment63");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(has_win_key(&scene_txt), "scene.json has win");
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["scene"], "lane");
    assert!(v.get("won").is_none(), "physics.json may omit won");
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    let won = next.get("won").expect("next-physics.json has won");
    assert_eq!(won["kind"], "delivered");
    assert_eq!(won["body"], "token");
    assert_eq!(won["scene"], "courtyard");
    let next_frame = out.join("next-frame.png");
    assert!(next_frame.is_file(), "next-frame.png must exist");
    assert!(fs::metadata(&next_frame).unwrap().len() > 256);
}
