use crate::{
    orbit_transform::{OrbitTransform, Smoother},
    polar_direction::PolarDirection,
};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    render::prelude::*,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

#[derive(Bundle)]
pub struct UnrealCameraBundle {
    controller: UnrealCameraController,
    #[bundle]
    perspective: PerspectiveCameraBundle,
}

impl UnrealCameraBundle {
    pub fn new(
        mut perspective: PerspectiveCameraBundle,
        control_config: UnrealCameraControlConfig,
        eye: Vec3,
        target: Vec3,
    ) -> Self {
        // Make sure the transform is consistent with the controller to start.
        perspective.transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);

        Self {
            perspective,
            controller: UnrealCameraController::new(control_config, eye, target),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct UnrealCameraControlConfig {
    pub mouse_rotate_sensitivity: f32,
    pub mouse_translate_sensitivity: f32,
    pub trackpad_translate_sensitivity: f32,
    pub smoothing_weight: f32,
}

impl Default for UnrealCameraControlConfig {
    fn default() -> Self {
        Self {
            mouse_rotate_sensitivity: 0.002,
            mouse_translate_sensitivity: 0.1,
            trackpad_translate_sensitivity: 0.1,
            smoothing_weight: 0.9,
        }
    }
}

/// A camera controlled with the mouse in the same way as Unreal Engine's viewport controller.
pub struct UnrealCameraController {
    control_config: UnrealCameraControlConfig,
    transform: OrbitTransform,
    smoother: Smoother,
    enabled: bool,
}

impl UnrealCameraController {
    pub fn new(control_config: UnrealCameraControlConfig, pivot: Vec3, orbit: Vec3) -> Self {
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

pub enum ControlEvent {
    Locomotion(Vec2),
    Rotate(Vec2),
    Translate(Vec2),
}

pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    controllers: Query<&UnrealCameraController>,
) {
    let camera = if let Some(camera) = controllers.iter().next() {
        camera
    } else {
        return;
    };
    let UnrealCameraController {
        control_config,
        enabled,
        ..
    } = &*camera;

    if !*enabled {
        return;
    }

    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        mouse_delta += event.delta;
    }

    match (
        mouse_buttons.pressed(MouseButton::Left),
        mouse_buttons.pressed(MouseButton::Right),
    ) {
        (true, true) => {
            events.send(ControlEvent::Translate(
                control_config.mouse_translate_sensitivity * mouse_delta,
            ));
        }
        (true, false) => {
            events.send(ControlEvent::Locomotion(Vec2::new(
                control_config.mouse_rotate_sensitivity * mouse_delta.x,
                control_config.mouse_translate_sensitivity * mouse_delta.y,
            )));
        }
        (false, true) => {
            events.send(ControlEvent::Rotate(
                control_config.mouse_rotate_sensitivity * mouse_delta,
            ));
        }
        _ => (),
    }

    // On Mac, mouse wheel is the trackpad, treated the same as both mouse buttons down.
    let mut trackpad_delta = Vec2::ZERO;
    for event in mouse_wheel_reader.iter() {
        trackpad_delta.x += event.x;
        trackpad_delta.y += event.y;
    }
    events.send(ControlEvent::Translate(
        control_config.trackpad_translate_sensitivity * trackpad_delta,
    ));
}

pub fn control_system(
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&mut UnrealCameraController, &mut Transform)>,
) {
    let (mut camera, mut scene_tfm) = if let Some((camera, tfm)) = cameras.iter_mut().next() {
        (camera, tfm)
    } else {
        return;
    };
    let UnrealCameraController {
        control_config,
        transform,
        smoother,
        enabled,
    } = &mut *camera;

    if *enabled {
        let look_vector = transform.pivot_to_orbit_direction();
        let mut polar_vector = PolarDirection::from_vector(look_vector);
        let forward_vector = Vec3::new(look_vector.x, 0.0, look_vector.z).normalize();

        let yaw_rot = Quat::from_axis_angle(Vec3::Y, polar_vector.get_yaw());
        let rot_x = yaw_rot * Vec3::X;
        let rot_y = yaw_rot * Vec3::Y;

        for event in events.iter() {
            match event {
                ControlEvent::Locomotion(delta) => {
                    // Translates forward/backward and rotates about the Y axis.
                    polar_vector.add_yaw(-delta.x);
                    transform.pivot -= delta.y * forward_vector;
                }
                ControlEvent::Rotate(delta) => {
                    // Rotates with pitch and yaw.
                    polar_vector.add_yaw(-delta.x);
                    polar_vector.add_pitch(-delta.y);
                }
                ControlEvent::Translate(delta) => {
                    // Translates up/down (Y) and left/right (X).
                    transform.pivot -= delta.x * rot_x + delta.y * rot_y;
                }
            }
        }

        polar_vector.assert_not_looking_up();

        transform.set_orbit_in_direction(polar_vector.unit_vector());
    } else {
        events.iter(); // Drop the events.
    }

    *scene_tfm = smoother
        .smooth_transform(control_config.smoothing_weight, transform)
        .pivot_look_at_orbit_transform();
}
