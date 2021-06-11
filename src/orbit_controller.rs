use crate::{
    orbit_transform::{OrbitTransform, Smoother},
    polar_direction::PolarDirection,
};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{mouse::MouseMotion, prelude::*},
    math::prelude::*,
    render::prelude::*,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

#[derive(Bundle)]
pub struct OrbitCameraBundle {
    controller: OrbitCameraController,
    #[bundle]
    perspective: PerspectiveCameraBundle,
}

impl OrbitCameraBundle {
    pub fn new(
        mut perspective: PerspectiveCameraBundle,
        control_config: OrbitCameraControlConfig,
        eye: Vec3,
        target: Vec3,
    ) -> Self {
        // Make sure the transform is consistent with the controller to start.
        perspective.transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);

        Self {
            perspective,
            controller: OrbitCameraController::new(control_config, target, eye),
        }
    }
}

pub enum ControlEvent {
    Rotate(Vec2),
    Translate(Vec2),
}

pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        mouse_delta += event.delta;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        if keyboard.pressed(KeyCode::LControl) {
            events.send(ControlEvent::Rotate(mouse_delta));
        } else {
            events.send(ControlEvent::Translate(mouse_delta));
        }
    }
}

pub struct OrbitCameraController {
    control_config: OrbitCameraControlConfig,
    transform: OrbitTransform,
    smoother: Smoother,
    enabled: bool,
}

impl OrbitCameraController {
    pub fn new(control_config: OrbitCameraControlConfig, pivot: Vec3, orbit: Vec3) -> Self {
        Self {
            control_config,
            transform: OrbitTransform { pivot, orbit },
            smoother: Default::default(),
            enabled: true,
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct OrbitCameraControlConfig {
    pub mouse_rotate_sensitivity: f32,
    pub mouse_translate_sensitivity: f32,
    pub smoothing_weight: f32,
}

impl Default for OrbitCameraControlConfig {
    fn default() -> Self {
        Self {
            mouse_rotate_sensitivity: 0.002,
            mouse_translate_sensitivity: 0.1,
            smoothing_weight: 0.8,
        }
    }
}

pub fn control_system(
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&mut OrbitCameraController, &mut Transform)>,
) {
    let (mut camera, mut scene_tfm) = if let Some((camera, tfm)) = cameras.iter_mut().next() {
        (camera, tfm)
    } else {
        return;
    };
    let OrbitCameraController {
        control_config,
        transform,
        smoother,
        enabled,
    } = &mut *camera;

    if *enabled {
        let mut polar_vector = PolarDirection::from_vector(transform.pivot_to_orbit_direction());

        for event in events.iter() {
            match event {
                ControlEvent::Rotate(delta) => {
                    polar_vector.add_yaw(-control_config.mouse_rotate_sensitivity * delta.x);
                    polar_vector.add_pitch(control_config.mouse_rotate_sensitivity * delta.y);
                    polar_vector.assert_not_looking_up();
                }
                ControlEvent::Translate(delta) => {
                    let right_dir = scene_tfm.rotation * -Vec3::X;
                    let up_dir = scene_tfm.rotation * Vec3::Y;
                    transform.pivot += (delta.x * right_dir + delta.y * up_dir)
                        * control_config.mouse_translate_sensitivity;
                }
            }
        }

        transform.set_orbit_in_direction(polar_vector.unit_vector());
    } else {
        events.iter(); // Drop the events.
    }

    *scene_tfm = smoother
        .smooth_transform(control_config.smoothing_weight, transform)
        .orbit_look_at_pivot_transform();
}
