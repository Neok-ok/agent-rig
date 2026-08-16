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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub dt: f32,
    pub frame_stride: u32,
    pub frames: Vec<TrajectoryFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryFrame {
    pub step: u32,
    pub bodies: Vec<PhysicsBodyState>,
    pub contacts: Vec<PhysicsContact>,
}

struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    body_handles: HashMap<String, RigidBodyHandle>,
    collider_to_id: HashMap<ColliderHandle, String>,
    body_ids: Vec<String>,
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl PhysicsWorld {
    fn from_scene(scene: &Scene, dt: f32) -> Self {
        let mut world = Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            body_handles: HashMap::new(),
            collider_to_id: HashMap::new(),
            body_ids: Vec::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: {
                let mut p = IntegrationParameters::default();
                p.dt = dt;
                p
            },
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        };
        world.populate(scene);
        world
    }

    fn populate(&mut self, scene: &Scene) {

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
                .build()
        };
        let handle = self.rigid_body_set.insert(rb);

        // Density from authored mass so inertia is non-zero (needed for body-body hits).
        let collider = match body.shape {
            Shape::Box { size } => {
                let mut b = ColliderBuilder::cuboid(size[0] * 0.5, size[1] * 0.5, size[2] * 0.5)
                    .friction(0.35)
                    .restitution(0.05);
                if body.mass > 0.0 {
                    let vol = (size[0] * size[1] * size[2]).max(1e-8);
                    b = b.density(body.mass / vol);
                }
                b.build()
            }
            Shape::Sphere { radius } => {
                let mut b = ColliderBuilder::ball(radius).friction(0.25).restitution(0.05);
                if body.mass > 0.0 {
                    let vol = (4.0 / 3.0 * std::f32::consts::PI * radius * radius * radius).max(1e-8);
                    b = b.density(body.mass / vol);
                }
                b.build()
            }
        };
        let ch = self.collider_set.insert_with_parent(collider, handle, &mut self.rigid_body_set);
            self.body_handles.insert(body.id.clone(), handle);
            self.collider_to_id.insert(ch, body.id.clone());
            self.body_ids.push(body.id.clone());
        }
    }

    fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    fn snapshot_bodies(&self) -> Vec<PhysicsBodyState> {
        let mut bodies = Vec::new();
        for id in &self.body_ids {
            let handle = self.body_handles[id];
            let rb = &self.rigid_body_set[handle];
            let t = rb.translation();
            let r = rb.rotation();
            let lv = rb.linvel();
            let av = rb.angvel();
            bodies.push(PhysicsBodyState {
                id: id.clone(),
                position: [t.x, t.y, t.z],
                rotation_wxyz: [r.w, r.i, r.j, r.k],
                linear_velocity: [lv.x, lv.y, lv.z],
                angular_velocity: [av.x, av.y, av.z],
            });
        }
        bodies
    }

    fn snapshot_contacts(&self) -> Vec<PhysicsContact> {
        let mut contacts = Vec::new();
        for pair in self.narrow_phase.contact_pairs() {
            if !pair.has_any_active_contact {
                continue;
            }
            let Some(id_a) = self.collider_to_id.get(&pair.collider1) else {
                continue;
            };
            let Some(id_b) = self.collider_to_id.get(&pair.collider2) else {
                continue;
            };
            let c1 = &self.collider_set[pair.collider1];

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
        contacts
    }

    fn snapshot_frame(&self, step: u32) -> TrajectoryFrame {
        TrajectoryFrame {
            step,
            bodies: self.snapshot_bodies(),
            contacts: self.snapshot_contacts(),
        }
    }
}

pub fn step_physics(scene: &Scene, steps: u32, dt: f32) -> Result<PhysicsDump, String> {
    let mut world = PhysicsWorld::from_scene(scene, dt);
    for _ in 0..steps {
        world.step();
    }
    Ok(PhysicsDump {
        steps,
        dt,
        gravity: [0.0, -9.81, 0.0],
        bodies: world.snapshot_bodies(),
        contacts: world.snapshot_contacts(),
    })
}

/// Step physics and record a snapshot every `frame_stride` steps, including step 0.
/// `frame_count` is the number of snapshots (and later, PNGs), not the physics step count.
pub fn simulate_trajectory(
    scene: &Scene,
    frame_count: u32,
    frame_stride: u32,
    dt: f32,
) -> Result<Trajectory, String> {
    if frame_count == 0 {
        return Err("frame_count must be ≥ 1".into());
    }
    if frame_stride == 0 {
        return Err("frame_stride must be ≥ 1".into());
    }
    let mut world = PhysicsWorld::from_scene(scene, dt);
    let mut frames = Vec::with_capacity(frame_count as usize);
    frames.push(world.snapshot_frame(0));
    for i in 1..frame_count {
        for _ in 0..frame_stride {
            world.step();
        }
        frames.push(world.snapshot_frame(i * frame_stride));
    }
    Ok(Trajectory {
        dt,
        frame_stride,
        frames,
    })
}
