use crate::{LookAngles, LookTransform, LookTransformBundle, Smoother};

use bevy::{
    app::prelude::*,
    ecs::prelude::*,
    input::{mouse::MouseMotion, prelude::*},
    math::prelude::*,
    prelude::ReflectDefault,
    reflect::Reflect,
    time::Time,
    transform::components::Transform,
};

#[derive(Default)]
pub struct FpsCameraPlugin {
    pub override_input_system: bool,
}

impl FpsCameraPlugin {
    pub fn new(override_input_system: bool) -> Self {
        Self {
            override_input_system,
        }
    }
}

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .add_systems(PreUpdate, on_controller_enabled_changed)
            .add_systems(Update, control_system)
            .add_event::<ControlEvent>();

        if !self.override_input_system {
            app.add_systems(Update, default_input_map);
        }
    }
}

#[derive(Bundle)]
pub struct FpsCameraBundle {
    controller: FpsCameraController,
    look_transform: LookTransformBundle,
    transform: Transform,
}

impl FpsCameraBundle {
    pub fn new(controller: FpsCameraController, eye: Vec3, target: Vec3, up: Vec3) -> Self {
        // Make sure the transform is consistent with the controller to start.
        let transform = Transform::from_translation(eye).looking_at(target, up);

        Self {
            controller,
            look_transform: LookTransformBundle {
                transform: LookTransform::new(eye, target, up),
                smoother: Smoother::new(controller.smoothing_weight),
            },
            transform,
        }
    }
}

/// Your typical first-person camera controller.
#[derive(Clone, Component, Copy, Debug, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[reflect(Component, Default, Debug)]
pub struct FpsCameraController {
    pub enabled: bool,
    pub mouse_rotate_sensitivity: Vec2,
    pub translate_sensitivity: f32,
    pub smoothing_weight: f32,
}

impl Default for FpsCameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            mouse_rotate_sensitivity: Vec2::splat(0.2),
            translate_sensitivity: 2.0,
            smoothing_weight: 0.9,
        }
    }
}

#[derive(Event)]
pub enum ControlEvent {
    Rotate(Vec2),
    TranslateEye(Vec3),
}

define_on_controller_enabled_changed!(FpsCameraController);

pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    controllers: Query<&FpsCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let FpsCameraController {
        translate_sensitivity,
        mouse_rotate_sensitivity,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    events.send(ControlEvent::Rotate(
        mouse_rotate_sensitivity * cursor_delta,
    ));

    for (key, dir) in [
        (KeyCode::KeyW, Vec3::Z),
        (KeyCode::KeyA, Vec3::X),
        (KeyCode::KeyS, -Vec3::Z),
        (KeyCode::KeyD, -Vec3::X),
        (KeyCode::ShiftLeft, -Vec3::Y),
        (KeyCode::Space, Vec3::Y),
    ]
    .iter()
    .cloned()
    {
        if keyboard.pressed(key) {
            events.send(ControlEvent::TranslateEye(translate_sensitivity * dir));
        }
    }
}

pub fn control_system(
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&FpsCameraController, &mut LookTransform)>,
    time: Res<Time>,
) {
    // Can only control one camera at a time.
    let mut transform = if let Some((_, transform)) = cameras.iter_mut().find(|c| c.0.enabled) {
        transform
    } else {
        return;
    };

    let look_vector = transform.look_direction().unwrap();
    let mut look_angles = LookAngles::from_vector(look_vector);

    let yaw_rot = Quat::from_axis_angle(Vec3::Y, look_angles.get_yaw());
    let rot_x = yaw_rot * Vec3::X;
    let rot_y = yaw_rot * Vec3::Y;
    let rot_z = yaw_rot * Vec3::Z;

    let dt = time.delta_seconds();
    for event in events.read() {
        match event {
            ControlEvent::Rotate(delta) => {
                // Rotates with pitch and yaw.
                look_angles.add_yaw(dt * -delta.x);
                look_angles.add_pitch(dt * -delta.y);
            }
            ControlEvent::TranslateEye(delta) => {
                // Translates up/down (Y) left/right (X) and forward/back (Z).
                transform.eye += dt * delta.x * rot_x + dt * delta.y * rot_y + dt * delta.z * rot_z;
            }
        }
    }

    look_angles.assert_not_looking_up();

    transform.target = transform.eye + transform.radius() * look_angles.unit_vector();
}
