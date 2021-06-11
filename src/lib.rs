mod orbit_controller;
mod orbit_transform;
mod polar_direction;
mod unreal_controller;

pub use orbit_controller::{OrbitCameraBundle, OrbitCameraControlConfig, OrbitCameraController};
pub use orbit_transform::*;
pub use polar_direction::PolarDirection;
pub use unreal_controller::{
    UnrealCameraBundle, UnrealCameraControlConfig, UnrealCameraController,
};

use bevy::{app::prelude::*, ecs::prelude::*};

pub struct SmoothCamerasPlugin;

impl Plugin for SmoothCamerasPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(unreal_controller::default_input_map.system())
            .add_system(unreal_controller::control_system.system())
            .add_system(orbit_controller::default_input_map.system())
            .add_system(orbit_controller::control_system.system())
            .add_event::<orbit_controller::ControlEvent>()
            .add_event::<unreal_controller::ControlEvent>();
    }
}
