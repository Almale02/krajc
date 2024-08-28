use bevy_ecs::component::Component;
use rapier3d::geometry::{
    Collider as Coll, ColliderBuilder, ColliderHandle as CollHandle, SharedShape,
};

#[derive(Component)]
pub struct ColliderHandle(pub CollHandle);

#[derive(Component)]
pub struct Collider(pub Coll);

impl Collider {
    pub fn new(shape: SharedShape) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: ColliderBuilder::new(shape),
        }
    }
}

pub struct CustomColliderBuilder {
    builder: ColliderBuilder,
}

impl CustomColliderBuilder {
    /// The set of active collision types for this collider.
    pub fn active_collision_types(
        self,
        active_collision_types: rapier3d::prelude::ActiveCollisionTypes,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.active_collision_types(active_collision_types),
        }
    }

    /// The set of events enabled for this collider.
    pub fn active_events(
        self,
        active_events: rapier3d::prelude::ActiveEvents,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.active_events(active_events),
        }
    }

    /// The set of physics hooks enabled for this collider.
    pub fn active_hooks(
        self,
        active_hooks: rapier3d::prelude::ActiveHooks,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.active_hooks(active_hooks),
        }
    }

    // Finishes the collider builder and returns the collider that you can add to you entities
    pub fn build(&self) -> Collider {
        Collider(self.builder.build())
    }

    /// Sets the collision groups used by this collider.
    ///
    /// Two colliders will interact iff. their collision groups are compatible.
    /// See [InteractionGroups::test] for details.
    pub fn collision_groups(
        self,
        groups: rapier3d::prelude::InteractionGroups,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.collision_groups(groups),
        }
    }

    /// Sets the total force magnitude beyond which a contact force event can be emitted.
    pub fn contact_force_event_threshold(
        self,
        threshold: rapier3d::prelude::Real,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.contact_force_event_threshold(threshold),
        }
    }

    /// Sets the contact skin of the collider.
    ///
    /// The contact skin acts as if the collider was enlarged with a skin of width `skin_thickness`
    /// around it, keeping objects further apart when colliding.
    ///
    /// A non-zero contact skin can increase performance, and in some cases, stability. However
    /// it creates a small gap between colliding object (equal to the sum of their skin). If the
    /// skin is sufficiently small, this might not be visually significant or can be hidden by the
    /// rendering assets.
    pub fn contact_skin(self, skin_thickness: rapier3d::prelude::Real) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.contact_skin(skin_thickness),
        }
    }

    /// Sets the uniform density of the collider this builder will build.
    ///
    /// This will be overridden by a call to [`Self::mass`] or [`Self::mass_properties`] so it only
    /// makes sense to call either [`Self::density`] or [`Self::mass`] or [`Self::mass_properties`].
    ///
    /// The mass and angular inertia of this collider will be computed automatically based on its
    /// shape.
    pub fn density(self, density: rapier3d::prelude::Real) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.density(density),
        }
    }

    /// Enable or disable the collider after its creation.
    pub fn enabled(self, enabled: bool) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.enabled(enabled),
        }
    }

    /// Sets the friction coefficient of the collider this builder will build.
    pub fn friction(self, friction: rapier3d::prelude::Real) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.friction(friction),
        }
    }

    /// Sets the rule to be used to combine two friction coefficients in a contact.
    pub fn friction_combine_rule(
        self,
        rule: rapier3d::prelude::CoefficientCombineRule,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.friction_combine_rule(rule),
        }
    }

    /// Sets the mass of the collider this builder will build.
    ///
    /// This will be overridden by a call to [`Self::density`] or [`Self::mass_properties`] so it only
    /// makes sense to call either [`Self::density`] or [`Self::mass`] or [`Self::mass_properties`].
    ///
    /// The angular inertia of this collider will be computed automatically based on its shape
    /// and this mass value.
    pub fn mass(self, mass: rapier3d::prelude::Real) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.mass(mass),
        }
    }

    /// Sets the mass properties of the collider this builder will build.
    ///
    /// This will be overridden by a call to [`Self::density`] or [`Self::mass`] so it only
    /// makes sense to call either [`Self::density`] or [`Self::mass`] or [`Self::mass_properties`].
    pub fn mass_properties(
        self,
        mass_properties: rapier3d::prelude::MassProperties,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.mass_properties(mass_properties),
        }
    }

    /// Sets the initial position (translation and orientation) of the collider to be created.
    ///
    /// If the collider will be attached to a rigid-body, this sets the position relative
    /// to the rigid-body it will be attached to.
    pub fn position(
        self,
        pos: rapier3d::prelude::Isometry<rapier3d::prelude::Real>,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.position(pos),
        }
    }

    /// Sets the restitution coefficient of the collider this builder will build.
    pub fn restitution(self, restitution: rapier3d::prelude::Real) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.restitution(restitution),
        }
    }

    /// Sets the rule to be used to combine two restitution coefficients in a contact.
    pub fn restitution_combine_rule(
        self,
        rule: rapier3d::prelude::CoefficientCombineRule,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.restitution_combine_rule(rule),
        }
    }

    /// Sets the initial orientation of the collider to be created.
    ///
    /// If the collider will be attached to a rigid-body, this sets the orientation relative to the
    /// rigid-body it will be attached to.
    pub fn rotation(
        self,
        angle: rapier3d::prelude::AngVector<rapier3d::prelude::Real>,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.rotation(angle),
        }
    }

    /// Sets whether or not the collider built by this builder is a sensor.
    pub fn sensor(self, is_sensor: bool) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.sensor(is_sensor),
        }
    }

    /// Sets the solver groups used by this collider.
    ///
    /// Forces between two colliders in contact will be computed iff their solver groups are
    /// compatible. See [InteractionGroups::test] for details.
    pub fn solver_groups(
        self,
        groups: rapier3d::prelude::InteractionGroups,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.solver_groups(groups),
        }
    }

    /// Sets the initial translation of the collider to be created.
    ///
    /// If the collider will be attached to a rigid-body, this sets the translation relative to the
    /// rigid-body it will be attached to.
    pub fn translation(
        self,
        translation: rapier3d::prelude::Vector<rapier3d::prelude::Real>,
    ) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.translation(translation),
        }
    }

    /// Sets an arbitrary user-defined 128-bit integer associated to the colliders built by this builder.
    pub fn user_data(self, data: u128) -> CustomColliderBuilder {
        CustomColliderBuilder {
            builder: self.builder.user_data(data),
        }
    }
}

fn test() {
    let a = Collider::new(SharedShape::ball(5.))
        .friction(4.)
        .restitution(2.)
        .density(10.)
        .user_data(432)
        .build();
}
