pub mod physics;
pub mod components;

pub mod prelude {
    pub use crate::physics::physics_workload;
}

pub use components::*;
pub use tetra;
pub use shipyard;
pub use physics::prelude::*;