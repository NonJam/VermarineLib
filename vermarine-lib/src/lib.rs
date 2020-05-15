pub mod physics;
pub mod components;
pub mod input;
pub mod starter;

pub mod prelude {
    pub use crate::physics::physics_workload;
}

pub use physics::physics_workload;
pub use components::*;
pub use physics::CollisionBody;
pub use physics::CollisionShape;
pub use physics::Collider;
pub use physics::Collision;

pub use tetra;
pub use shipyard;