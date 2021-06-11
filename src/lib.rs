//! A collection of exponentially-smoothed camera controllers for the Bevy Engine.
//!
//! All controllers are based on a `LookTransform` component, which is just an `eye`
//! point that looks at a `target` point. By modifying this component, the scene
//! graph `Transform` will automatically be synchronized.
//!
//! A `LookTransform` can be smoothed by adding a `Smoother` component, and the
//! smoothing will happen automatically.
//!
//! ```rust
//! // Enables the system that synchronizes your `Transform`s and `LookTransform`s.
//! app.add_plugin(LookTransformPlugin);
//!
//! ...
//!
//! commands
//!     .spawn_bundle(LookTransformBundle {
//!         transform: LookTransform { eye, target },
//!         smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
//!     })
//! ```
//!
//! # Built-In Controllers
//!
//! These plugins depend on the `LookTransformPlugin`:
//!
//! - `UnrealCameraPlugin + UnrealCameraBundle`
//! - `OrbitCameraPlugin + OrbitCameraBundle`

pub mod controllers;

mod look_transform;
mod polar_direction;

pub use look_transform::*;
pub use polar_direction::*;
