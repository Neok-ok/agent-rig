use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::scene::{Scene, Shape};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsDump {
    pub steps: u32,
    pub dt: f32,
    pub gravity: [f32; 3],
    pub bodies: Vec<PhysicsBodyState>,
    pub contacts: Vec<PhysicsContact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBodyState {
    pub id: String,
    pub position: [f32; 3],
    pub rotation_wxyz: [f32; 4],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsContact {
    pub body_a: String,
    pub body_b: String,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

pub fn step_physics(scene: &Scene, steps: u32, dt: f32) -> Result<PhysicsDump, String> {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    let mut body_handles: HashMap<String, RigidBodyHandle> = HashMap::new();
    let mut collider_to_id: HashMap<ColliderHandle, String> = HashMap::new();

    for body in &scene.bodies {
        let [x, y, z] = body.position;
        let [w, qx, qy, qz] = body.rotation_wxyz;
        let rotation = nalgebra::UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(w, qx, qy, qz));
        let iso = Isometry::from_parts(Translation::new(x, y, z), rotation);

        let rb = if body.mass <= 0.0 {
            RigidBodyBuilder::fixed().position(iso).build()
        } else {
            let [vx, vy, vz] = body.linear_velocity;
            RigidBodyBuilder::dynamic()
                .position(iso)
                .linvel(vector![vx, vy, vz])
                .additional_mass(body.mass)
                .build()
        };
        let handle = rigid_body_set.insert(rb);

        let collider = match body.shape {
            Shape::Box { size } => ColliderBuilder::cuboid(size[0] * 0.5, size[1] * 0.5, size[2] * 0.5)
                .friction(0.8)
                .restitution(0.05)
                .density(0.0)
                .build(),
            Shape::Sphere { radius } => ColliderBuilder::ball(radius)
                .friction(0.6)
                .restitution(0.05)
                .density(0.0)
                .build(),
        };
        let ch = collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);
        body_handles.insert(body.id.clone(), handle);
        collider_to_id.insert(ch, body.id.clone());
    }

    let gravity = vector![0.0, -9.81, 0.0];
    let mut integration_parameters = IntegrationParameters::default();
    integration_parameters.dt = dt;

    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let mut query_pipeline = QueryPipeline::new();

    for _ in 0..steps {
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            Some(&mut query_pipeline),
            &(),
            &(),
        );
    }

    let mut bodies = Vec::new();
    for body in &scene.bodies {
        let handle = body_handles
            .get(&body.id)
            .ok_or_else(|| format!("missing body {}", body.id))?;
        let rb = &rigid_body_set[*handle];
        let t = rb.translation();
        let r = rb.rotation();
        let lv = rb.linvel();
        let av = rb.angvel();
        bodies.push(PhysicsBodyState {
            id: body.id.clone(),
            position: [t.x, t.y, t.z],
            rotation_wxyz: [r.w, r.i, r.j, r.k],
            linear_velocity: [lv.x, lv.y, lv.z],
            angular_velocity: [av.x, av.y, av.z],
        });
    }

    let mut contacts = Vec::new();
    for pair in narrow_phase.contact_pairs() {
        if !pair.has_any_active_contact {
            continue;
        }
        let Some(id_a) = collider_to_id.get(&pair.collider1) else {
            continue;
        };
        let Some(id_b) = collider_to_id.get(&pair.collider2) else {
            continue;
        };
        let c1 = &collider_set[pair.collider1];

        let mut pushed = false;
        for manifold in &pair.manifolds {
            for sc in &manifold.data.solver_contacts {
                let p = sc.point;
                let n = manifold.data.normal;
                contacts.push(PhysicsContact {
                    body_a: id_a.clone(),
                    body_b: id_b.clone(),
                    point: [p.x, p.y, p.z],
                    normal: [n.x, n.y, n.z],
                });
                pushed = true;
            }
            if !pushed {
                for contact in &manifold.points {
                    let p = c1.position() * contact.local_p1;
                    let n = c1.position() * manifold.local_n1;
                    contacts.push(PhysicsContact {
                        body_a: id_a.clone(),
                        body_b: id_b.clone(),
                        point: [p.x, p.y, p.z],
                        normal: [n.x, n.y, n.z],
                    });
                    pushed = true;
                }
            }
        }
    }

    Ok(PhysicsDump {
        steps,
        dt,
        gravity: [0.0, -9.81, 0.0],
        bodies,
        contacts,
    })
}
