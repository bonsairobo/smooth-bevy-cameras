use crate::{LookAngles, LookTransform, LookTransformBundle, Smoother};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    time::Time,
    transform::components::Transform, prelude::Projection,
};

#[derive(Default)]
pub struct OrbitCameraPlugin {
    pub override_input_system: bool,
}

impl OrbitCameraPlugin {
    pub fn new(override_input_system: bool) -> Self {
        Self {
            override_input_system,
        }
    }
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .add_system(setup_orthographic_transform)
            .add_system(on_controller_enabled_changed.in_base_set(CoreSet::PreUpdate))
            .add_system(control_system)
            .add_event::<ControlEvent>();

        if !self.override_input_system {
            app.add_system(default_input_map);
        }
    }
}

#[derive(Bundle)]
pub struct OrbitCameraBundle {
    controller: OrbitCameraController,
    #[bundle]
    look_transform: LookTransformBundle,
    transform: Transform,
}

impl OrbitCameraBundle {
    pub fn new(controller: OrbitCameraController, eye: Vec3, target: Vec3, up: Vec3) -> Self {
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

/// A 3rd person camera that orbits around the target.
#[derive(Clone, Component, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OrbitCameraController {
    pub enabled: bool,
    pub mouse_rotate_sensitivity: Vec2,
    pub mouse_translate_sensitivity: Vec2,
    pub mouse_wheel_zoom_sensitivity: f32,
    pub pixels_per_line: f32,
    pub smoothing_weight: f32,
    pub ortho_setup: bool,
}

impl Default for OrbitCameraController {
    fn default() -> Self {
        Self {
            mouse_rotate_sensitivity: Vec2::splat(0.08),
            mouse_translate_sensitivity: Vec2::splat(0.1),
            mouse_wheel_zoom_sensitivity: 0.2,
            smoothing_weight: 0.8,
            enabled: true,
            pixels_per_line: 53.0,
            ortho_setup: false
        }
    }
}

pub enum ControlEvent {
    Orbit(Vec2),
    TranslateTarget(Vec2),
    Zoom(f32),
}

define_on_controller_enabled_changed!(OrbitCameraController);

fn setup_orthographic_transform(
    mut cameras: Query<(&mut OrbitCameraController, &mut LookTransform, &Projection)>,
) {
    let (mut cc, mut transform, projection) =
        if let Some((cc, transform, projection)) = cameras.iter_mut().find(|c| c.0.enabled) {
            (cc, transform, projection)
        } else {
            return;
        };
    if cc.ortho_setup { return; }
    else { cc.ortho_setup = true; }

    let scale = if let Projection::Orthographic(ref o) = projection {
        o.scale
    } else {
        0.0
    };
    transform.scale = 2.0;
}

pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        mouse_wheel_zoom_sensitivity,
        pixels_per_line,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    if keyboard.pressed(KeyCode::LControl) {
        events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        events.send(ControlEvent::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta,
        ));
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.iter() {
        // scale the event magnitude per pixel or per line
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / pixels_per_line,
        };
        scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
    }
    events.send(ControlEvent::Zoom(scalar));
}

pub fn control_system(
    time: Res<Time>,
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&OrbitCameraController, &mut LookTransform, &Transform, &Projection)>,
) {
    // Can only control one camera at a time.
    let (mut transform, scene_transform, projection) =
        if let Some((_, transform, scene_transform, proj)) = cameras.iter_mut().find(|c| c.0.enabled) {
            (transform, scene_transform, proj)
        } else {
            return;
        };

    let mut look_angles = LookAngles::from_vector(-transform.look_direction().unwrap());
    let mut radius_scalar = 1.0;
    let is_orthographic = matches!(projection, Projection::Orthographic(_));

    let dt = time.delta_seconds();
    for event in events.iter() {
        match event {
            ControlEvent::Orbit(delta) => {
                look_angles.add_yaw(dt * -delta.x);
                look_angles.add_pitch(dt * delta.y);
            }
            ControlEvent::TranslateTarget(delta) => {
                let right_dir = scene_transform.rotation * -Vec3::X;
                let up_dir = scene_transform.rotation * Vec3::Y;
                let mut translation = dt * delta.x * right_dir + dt * delta.y * up_dir;
                if is_orthographic {
                    let scale = transform.scale * 0.5;
                    translation *= scale;
                }
                transform.target += translation;
            }
            ControlEvent::Zoom(scalar) => {
                radius_scalar *= scalar;
            }
        }
    }

    look_angles.assert_not_looking_up();

    if is_orthographic {
        transform.scale *= radius_scalar;
        transform.eye = transform.target + transform.radius() * look_angles.unit_vector();
    } else {
        let new_radius = (radius_scalar * transform.radius())
            .min(1000000.0)
            .max(0.001);
        transform.eye = transform.target + new_radius * look_angles.unit_vector();
    }
}
