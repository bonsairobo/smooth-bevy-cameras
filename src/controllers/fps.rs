use crate::{LookAngles, LookTransform, LookTransformBundle, Smoother};
use bevy::window::{PrimaryWindow, Window};

use bevy::{
    app::prelude::*,
    ecs::prelude::*,
    input::{mouse::MouseMotion, prelude::*},
    math::prelude::*,
    prelude::ReflectDefault,
    reflect::Reflect,
    time::Time,
    transform::components::Transform,
    window::{CursorGrabMode, CursorOptions},
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
            .add_systems(Startup, init)
            .add_systems(Update, (control_system, reset_cursor_system))
            .add_message::<ControlMessage>();

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
    /// If set to true, the cursor will be locked and hidden when the camera is active.
    pub auto_hide_cursor: bool,
}

impl Default for FpsCameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            mouse_rotate_sensitivity: Vec2::splat(0.2),
            translate_sensitivity: 2.0,
            smoothing_weight: 0.9,
            auto_hide_cursor: true,
        }
    }
}

#[derive(Message)]
pub enum ControlMessage {
    Rotate(Vec2),
    TranslateEye(Vec3),
}

/// A marker component for tracking when the cursor needs to be reset next frame
#[derive(Component)]
struct ResetCursorNextFrame;

define_on_controller_enabled_changed!(FpsCameraController);

fn init(
    mut cursor_options: Single<&mut CursorOptions>,
    mut commands: Commands,
    cameras: Query<(Entity, &FpsCameraController)>,
) {
    // Set initial cursor state
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
    
    // Mark cursor to be reset in the next frame for any enabled camera
    for (camera_entity, controller) in cameras.iter() {
        if controller.enabled && controller.auto_hide_cursor {
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
            commands.entity(camera_entity).insert(ResetCursorNextFrame);
            break; // Only need to do this for one camera
        }
    }
}

pub fn default_input_map(
    mut messages: MessageWriter<ControlMessage>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_messages: MessageReader<MouseMotion>,
    controllers: Query<&FpsCameraController>,
    cursor_options: Single<&CursorOptions>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    
    // Check if cursor is currently locked using the CursorOptions resource
    let cursor_locked = cursor_options.grab_mode == CursorGrabMode::Locked;
    
    let FpsCameraController {
        translate_sensitivity,
        mouse_rotate_sensitivity,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    // Only process mouse motion if cursor is locked
    if cursor_locked {
        for event in mouse_motion_messages.read() {
            cursor_delta += event.delta;
        }
    }

    messages.write(ControlMessage::Rotate(
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
            messages.write(ControlMessage::TranslateEye(translate_sensitivity * dir));
        }
    }
}

pub fn control_system(
    mut commands: Commands,
    mut messages: MessageReader<ControlMessage>,
    mut cameras: Query<(Entity, &FpsCameraController, &mut LookTransform)>,
    mut cursor_options: Single<&mut CursorOptions>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Can only control one camera at a time.
    let Some((entity, controller, mut transform)) = cameras.iter_mut().find_map(|(e, c, t)| {
        c.enabled.then_some((e, c, t))
    }) else {
        return;
    };

    // Handle cursor locking
    if controller.auto_hide_cursor {
        if key_input.just_pressed(KeyCode::AltLeft) || key_input.just_pressed(KeyCode::AltRight) {
            // Release cursor
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        } else if key_input.just_released(KeyCode::AltLeft) || key_input.just_released(KeyCode::AltRight) {
            // Lock cursor and mark for reset
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
            
            // Mark cursor to be reset in the next frame
            commands.entity(entity).insert(ResetCursorNextFrame);
        }
    }

    let look_vector = transform.look_direction().unwrap();
    let mut look_angles = LookAngles::from_vector(look_vector);

    let yaw_rot = Quat::from_axis_angle(Vec3::Y, look_angles.get_yaw());
    let rot_x = yaw_rot * Vec3::X;
    let rot_y = yaw_rot * Vec3::Y;
    let rot_z = yaw_rot * Vec3::Z;

    let dt = time.delta_secs();
    for event in messages.read() {
        match event {
            ControlMessage::Rotate(delta) => {
                // Rotates with pitch and yaw.
                look_angles.add_yaw(dt * -delta.x);
                look_angles.add_pitch(dt * -delta.y);
            }
            ControlMessage::TranslateEye(delta) => {
                // Translates up/down (Y) left/right (X) and forward/back (Z).
                transform.eye += dt * delta.x * rot_x + dt * delta.y * rot_y + dt * delta.z * rot_z;
            }
        }
    }

    look_angles.assert_not_looking_up();

    transform.target = transform.eye + transform.radius() * look_angles.unit_vector();
}


/// System that resets the cursor position on the frame after locking
fn reset_cursor_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut camera_query: Query<(Entity, &mut ResetCursorNextFrame)>,
    mut commands: Commands,
) {
    for (entity, _) in camera_query.iter_mut() {
        if let Ok(mut window) = windows.single_mut() {
            let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
            let _ = window.set_cursor_position(Some(center));
        }
        
        // Remove the marker component so this only runs once
        commands.entity(entity).remove::<ResetCursorNextFrame>();
    }
}
