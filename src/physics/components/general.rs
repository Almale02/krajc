use bevy_ecs::component::Component;
use rapier3d::{
    dynamics::{self as rapier, RigidBody as RB, RigidBodyBuilder, RigidBodySet, RigidBodyType},
    na::Vector3,
};

use crate::rendering::systems::general::Transform;

/*#[derive(Component)]
pub enum RigidBody {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}*/

#[derive(Component)]
pub struct RigidBody(pub RB);
impl RigidBody {
    pub fn new(rb_type: RigidBodyType) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: RigidBodyBuilder::new(rb_type),
        }
    }
}

pub struct CustomRigidBodyBuilder {
    builder: RigidBodyBuilder,
}

impl CustomRigidBodyBuilder {
    pub fn additional_mass(self, mass: rapier3d::prelude::Real) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.additional_mass(mass),
        }
    }

    pub fn additional_mass_properties(
        self,
        mprops: rapier3d::prelude::MassProperties,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.additional_mass_properties(mprops),
        }
    }

    pub fn additional_solver_iterations(
        self,
        additional_iterations: usize,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder,
        }
        .additional_solver_iterations(additional_iterations)
    }

    pub fn angular_damping(self, factor: rapier3d::prelude::Real) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.angular_damping(factor),
        }
    }

    pub fn angvel(
        self,
        angvel: rapier3d::prelude::AngVector<rapier3d::prelude::Real>,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.angvel(angvel),
        }
    }

    pub fn build(&self) -> RigidBody {
        RigidBody(self.builder.build())
    }

    pub fn can_sleep(self, can_sleep: bool) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.can_sleep(can_sleep),
        }
    }

    pub fn ccd_enabled(self, enabled: bool) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.ccd_enabled(enabled),
        }
    }

    pub fn dominance_group(self, group: i8) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.dominance_group(group),
        }
    }

    pub fn enabled(self, enabled: bool) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.enabled(enabled),
        }
    }

    pub fn enabled_rotations(
        self,
        allow_rotations_x: bool,
        allow_rotations_y: bool,
        allow_rotations_z: bool,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.enabled_rotations(
                allow_rotations_x,
                allow_rotations_y,
                allow_rotations_z,
            ),
        }
    }

    pub fn enabled_translations(
        self,
        allow_translations_x: bool,
        allow_translations_y: bool,
        allow_translations_z: bool,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.enabled_translations(
                allow_translations_x,
                allow_translations_y,
                allow_translations_z,
            ),
        }
    }

    pub fn gravity_scale(self, scale_factor: rapier3d::prelude::Real) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.gravity_scale(scale_factor),
        }
    }

    pub fn linear_damping(self, factor: rapier3d::prelude::Real) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.linear_damping(factor),
        }
    }

    pub fn linvel(
        self,
        linvel: rapier3d::prelude::Vector<rapier3d::prelude::Real>,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.linvel(linvel),
        }
    }

    pub fn lock_rotations(self) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.lock_rotations(),
        }
    }

    pub fn lock_translations(self) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.lock_translations(),
        }
    }

    pub fn locked_axes(self, locked_axes: rapier3d::prelude::LockedAxes) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.locked_axes(locked_axes),
        }
    }

    pub fn position(
        self,
        pos: rapier3d::prelude::Isometry<rapier3d::prelude::Real>,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.position(pos),
        }
    }

    pub fn rotation(
        self,
        angle: rapier3d::prelude::AngVector<rapier3d::prelude::Real>,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.rotation(angle),
        }
    }

    pub fn sleeping(self, sleeping: bool) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.sleeping(sleeping),
        }
    }

    pub fn soft_ccd_prediction(
        self,
        prediction_distance: rapier3d::prelude::Real,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.soft_ccd_prediction(prediction_distance),
        }
    }

    pub fn translation(
        self,
        translation: rapier3d::prelude::Vector<rapier3d::prelude::Real>,
    ) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.translation(translation),
        }
    }

    pub fn user_data(self, data: u128) -> CustomRigidBodyBuilder {
        CustomRigidBodyBuilder {
            builder: self.builder.user_data(data),
        }
    }
}

#[derive(Component)]
pub struct FixedRigidBody;
#[derive(Component)]
pub struct RigidBodyHandle(pub rapier::RigidBodyHandle, &'static mut RigidBodySet);

impl RigidBodyHandle {
    pub fn new(handle: rapier::RigidBodyHandle, set: &'static mut RigidBodySet) -> Self {
        RigidBodyHandle(handle, set)
    }

    pub fn get_body(&mut self) -> &mut RB {
        self.1.get_mut(self.0).unwrap()
    }
}

impl std::ops::Deref for RigidBodyHandle {
    type Target = rapier::RigidBodyHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Component, Default)]
pub struct TargetKinematicTransform(pub Transform);

#[derive(Component, Default)]
pub struct TargetKinematicVelocity {
    pub lin_vel: Vector3<f32>,
    pub ang_vel: Vector3<f32>,
}

#[derive(Component, Default)]
pub struct PhysicsSyncDirectBodyModifications;

#[derive(Component, Default)]
pub struct PhysicsDontSyncRotation;

#[derive(Component, Default)]
pub struct LinearVelocity(pub Vector3<f32>);

#[derive(Component, Default)]
pub struct AngularVelocity(pub Vector3<f32>);
