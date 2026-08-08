use rapier3d::prelude::*;

pub struct PhysicsEngine {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.joint_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_engine_step() {
        let mut engine = PhysicsEngine::new();
        let rigid_body = RigidBodyBuilder::dynamic().translation(vector![0.0, 0.0, 0.0]).build();
        let handle = engine.rigid_body_set.insert(rigid_body);
        engine.rigid_body_set.get_mut(handle).unwrap().apply_force(vector![1.0, 0.0, 0.0], true);
        engine.step();
        let body = engine.rigid_body_set.get(handle).unwrap();
        assert!(body.translation().x > 0.0);
    }
}
