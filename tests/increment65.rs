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
    increment60_scene_json, increment61_scene_json, increment62_scene_json,
    increment63_scene, increment63_scene_json, increment64_scene, increment64_scene_json,
    increment65_scene, increment65_scene_json, parse_scene, run_increment65, step_physics,
    DEFAULT_DT, INCREMENT65_STEPS,
};

fn prior_scene_jsons() -> [(&'static str, &'static str); 47] {
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
        ("63", increment63_scene_json()),
        ("64", increment64_scene_json()),
    ]
}

fn has_uses_key(json: &str) -> bool {
    let v: serde_json::Value = serde_json::from_str(json).expect("parse scene json");
    v.get("uses").is_some()
}

#[test]
fn increment65_does_not_mutate_prior() {
    assert!(
        increment63_scene().uses.is_empty(),
        "increment63_scene must stay use-free"
    );
    assert!(
        increment64_scene().uses.is_empty(),
        "increment64_scene must stay use-free"
    );
    assert_eq!(
        increment64_scene().drops.len(),
        1,
        "increment64 still has drops"
    );
    for (name, json) in prior_scene_jsons() {
        let parsed = parse_scene(json).unwrap_or_else(|e| panic!("increment {name} parse: {e}"));
        assert!(
            parsed.uses.is_empty(),
            "increment {name} scene.uses must stay empty"
        );
        if name == "63" || name == "64" {
            assert!(
                !has_uses_key(json),
                "increment {name} JSON must stay without a uses key"
            );
        } else {
            assert!(
                !has_uses_key(json),
                "increment {name} JSON must stay without a uses key"
            );
        }
    }
    let inc65 = increment65_scene_json();
    assert!(has_uses_key(inc65), "increment65 json HAS uses");
    let v: serde_json::Value = serde_json::from_str(inc65).expect("inc65 json");
    assert_eq!(v["uses"][0]["body"], "token");
    assert_eq!(v["uses"][0]["trigger"], "exit");
    assert_eq!(v["uses"][0]["by"], "walker");
}

#[test]
fn increment65_scene_is_win_plus_use() {
    let scene = increment65_scene();
    assert_eq!(scene.id, "lane");
    let win = scene.win.as_ref().expect("win");
    assert_eq!(win.kind, "delivered");
    assert_eq!(win.body, "token");
    assert_eq!(scene.uses.len(), 1, "exactly one use");
    assert_eq!(scene.uses[0].body, "token");
    assert_eq!(scene.uses[0].trigger, "exit");
    assert_eq!(scene.uses[0].by, "walker");
    assert!(scene.drops.is_empty(), "drops must be empty");
    let tr = scene.transition.as_ref().expect("transition");
    assert_eq!(tr.to, "courtyard");
    assert!(scene.pickups.iter().any(|p| p.hold), "has hold pickup");
}

#[test]
fn increment63_next_still_won_no_used() {
    let art = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts/increment63/next-physics.json");
    assert!(art.is_file(), "increment63 next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&art).unwrap()).unwrap();
    let won = next.get("won").expect("increment63 next-physics still won");
    assert_eq!(won["kind"], "delivered");
    assert_eq!(won["body"], "token");
    assert_eq!(won["scene"], "courtyard");
    assert!(
        next.get("used").is_none(),
        "increment63 next-physics must have no used key"
    );
    assert!(
        next.get("lost").is_none(),
        "increment63 next-physics must have no lost key"
    );
    let bodies = next["bodies"].as_array().expect("next bodies");
    assert!(bodies.iter().any(|b| b["id"] == "token"), "HAS token");
}

#[test]
fn increment64_next_still_lost_no_used() {
    let art = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts/increment64/next-physics.json");
    assert!(art.is_file(), "increment64 next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&art).unwrap()).unwrap();
    let lost = next.get("lost").expect("increment64 next-physics still lost");
    assert_eq!(lost["kind"], "empty_handed");
    assert_eq!(lost["body"], "token");
    assert_eq!(lost["scene"], "courtyard");
    assert!(
        next.get("used").is_none(),
        "increment64 next-physics must have no used key"
    );
    assert!(
        next.get("won").is_none(),
        "increment64 next-physics must have no won key"
    );
    let bodies = next["bodies"].as_array().expect("next bodies");
    assert!(bodies.iter().all(|b| b["id"] != "token"), "NO token");
}

#[test]
fn increment65_lane_dump() {
    let dump = step_physics(&increment65_scene(), INCREMENT65_STEPS, DEFAULT_DT)
        .expect("increment65 physics");
    assert_eq!(dump.scene, "lane");
    assert!(
        (85..=115).contains(&dump.steps),
        "dump.steps 85..=115, got {}",
        dump.steps
    );
    let stopped = dump.stopped.as_ref().expect("dump.stopped");
    assert_eq!(stopped.kind, "entered");
    assert_eq!(stopped.body, "exit");
    assert_eq!(stopped.at_step, 99);
    let tr = dump.transition.as_ref().expect("dump.transition");
    assert_eq!(tr.to, "courtyard");
    assert_eq!(tr.at_step, 99);
    let held = dump
        .held
        .iter()
        .find(|h| h.id == "token")
        .expect("dump.held token");
    assert_eq!(held.by, "walker");
    assert_eq!(held.at_step, 66);
    assert!(dump.dropped.is_empty(), "dropped omitted");
    assert!(dump.won.is_none(), "lane dump.won omitted");
    assert!(dump.lost.is_none(), "lane dump.lost omitted");
    assert_eq!(dump.used.len(), 1, "used stamped once");
    assert_eq!(dump.used[0].body, "token");
    assert_eq!(dump.used[0].trigger, "exit");
    assert_eq!(dump.used[0].by, "walker");
    assert_eq!(dump.used[0].at_step, 99);
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
    let v = serde_json::to_value(&dump).expect("serialize dump");
    assert!(v.get("dropped").is_none(), "serialized lane dump omits dropped");
    assert!(v.get("won").is_none(), "serialized lane dump omits won");
    assert!(v.get("lost").is_none(), "serialized lane dump omits lost");
    let used = v.get("used").expect("serialized lane dump has used");
    assert_eq!(used[0]["body"], "token");
    assert_eq!(used[0]["trigger"], "exit");
    assert_eq!(used[0]["by"], "walker");
    assert_eq!(used[0]["at_step"], 99);
}

#[test]
fn increment65_next_won() {
    let out = PathBuf::from("target/test-increment65-next");
    let _ = fs::remove_dir_all(&out);
    let _paths = run_increment65(&out, INCREMENT65_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment65");
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    assert_eq!(next["steps"], 1);
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
    let held = next.get("held").expect("held token");
    assert_eq!(held[0]["id"], "token");
    assert_eq!(held[0]["by"], "walker");
    assert_eq!(held[0]["at_step"], 66);
    let won = next.get("won").expect("dump.won");
    assert_eq!(won["kind"], "delivered");
    assert_eq!(won["body"], "token");
    assert_eq!(won["scene"], "courtyard");
    assert!(next.get("lost").is_none(), "lost omitted");
    assert!(next.get("used").is_none(), "used omitted on courtyard");
}

#[test]
fn increment65_writes() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment65.sh");
    assert!(script.is_file(), "scripts/increment65.sh must exist");
    let three = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/increment65-threejs.sh");
    assert!(three.is_file(), "scripts/increment65-threejs.sh must exist");
    let out = PathBuf::from("target/test-increment65-artifacts");
    let _ = fs::remove_dir_all(&out);
    let paths = run_increment65(&out, INCREMENT65_STEPS, DEFAULT_DT, 200, 112)
        .expect("run_increment65");
    assert!(paths.scene.is_file() && paths.physics.is_file() && paths.frame.is_file());
    assert!(fs::metadata(&paths.frame).unwrap().len() > 256);
    let scene_txt = fs::read_to_string(&paths.scene).unwrap();
    assert!(has_uses_key(&scene_txt), "scene.json has uses");
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paths.physics).unwrap()).unwrap();
    assert_eq!(v["scene"], "lane");
    let used = v.get("used").expect("physics.json has used");
    assert_eq!(used[0]["body"], "token");
    assert_eq!(used[0]["trigger"], "exit");
    assert_eq!(used[0]["by"], "walker");
    assert_eq!(used[0]["at_step"], 99);
    let next_path = out.join("next-physics.json");
    assert!(next_path.is_file(), "next-physics.json must exist");
    let next: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&next_path).unwrap()).unwrap();
    assert_eq!(next["scene"], "courtyard");
    let won = next.get("won").expect("next-physics.json has won");
    assert_eq!(won["kind"], "delivered");
    assert_eq!(won["body"], "token");
    assert_eq!(won["scene"], "courtyard");
    assert!(next.get("used").is_none(), "next-physics.json omits used");
    let next_frame = out.join("next-frame.png");
    assert!(next_frame.is_file(), "next-frame.png must exist");
    assert!(fs::metadata(&next_frame).unwrap().len() > 256);
}
