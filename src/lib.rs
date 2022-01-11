//! [![crates.io](https://img.shields.io/crates/v/smooth_bevy_cameras)](https://crates.io/crates/smooth_bevy_cameras)
//! [![docs.rs](https://docs.rs/smooth-bevy-cameras/badge.svg)](https://docs.rs/smooth-bevy-cameras)
//!
//! A collection of exponentially-smoothed camera controllers for the Bevy Engine.
//!
//! # Look Transform
//!
//! All controllers are based on a `LookTransform` component, which is just an `eye` point that looks at a `target` point. By
//! modifying this component, the scene graph `Transform` will automatically be synchronized.
//!
//! Any entities with `{Transform, LookTransform, Smoother}` components will automatically have their `Transform` smoothed.
//! Smoothing will have no effect on the `LookTransform`, only the final `Transform` in the scene graph.
//!
//! ```rust
//! use bevy::prelude::*;
//! use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};
//!
//! fn main() {
//!     App::build()
//!         .add_plugins(DefaultPlugins)
//!         // Enables the system that synchronizes your `Transform`s and `LookTransform`s.
//!         .add_plugin(LookTransformPlugin)
//!         .add_startup_system(setup.system())
//!         .add_system(move_camera_system.system());
//! }
//!
//! fn setup(mut commands: Commands) {
//!     let eye = Vec3::default();
//!     let target = Vec3::default();
//!
//!     commands
//!         .spawn_bundle(LookTransformBundle {
//!             transform: LookTransform { eye, target },
//!             smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
//!         })
//!         .insert_bundle(PerspectiveCameraBundle::default());
//!
//! }
//!
//! fn move_camera_system(mut cameras: Query<&mut LookTransform>) {
//!     // Later, another system will update the `Transform` and apply smoothing automatically.
//!     for mut c in cameras.iter_mut() { c.target += Vec3::new(1.0, 1.0, 1.0); }
//! }
//! ```
//!
//! # Look Angles
//!
//! When implementing a camera controller, it's often useful to work directly with the angles (pitch and yaw) of your look
//! direction. You can do this with the `LookAngles` type:
//!
//! ```rust
//! use bevy::prelude::*;
//! use smooth_bevy_cameras::{
//!     LookAngles,
//!     LookTransform
//! };
//!
//! fn look_angles(mut transform: LookTransform, delta: Vec2) {
//!     let mut angles = LookAngles::from_vector(transform.look_direction());
//!     angles.add_pitch(delta.y);
//!     angles.add_yaw(delta.x);
//!     transform.target = transform.target + 1.0 * transform.radius() * angles.unit_vector();
//! }
//! ```
//!
//! This is how the built-in controllers implement rotation controls.
//!
//! # Built-In Controllers
//!
//! These plugins depend on the `LookTransformPlugin`:
//!
//! - `FpsCameraPlugin + FpsCameraBundle`
//!   - WASD: Translate on the XZ plane
//!   - Shift/Space: Translate along the Y axis
//!   - Mouse: Rotate camera
//! - `OrbitCameraPlugin + OrbitCameraBundle`
//!   - CTRL + mouse drag: Rotate camera
//!   - Right mouse drag: Pan camera
//!   - Mouse wheel: Zoom
//! - `UnrealCameraPlugin + UnrealCameraBundle`
//!   - Left mouse drag: Locomotion
//!   - Right mouse drag: Rotate camera
//!   - Left and Right mouse drag: Pan camera

pub mod controllers;

mod look_angles;
mod look_transform;

pub use look_angles::*;
pub use look_transform::*;
