use bevy::{
    app::prelude::*, ecs::prelude::*, math::prelude::*, prelude::ReflectDefault, reflect::Reflect,
    transform::components::Transform,
};

pub struct LookTransformPlugin;

impl Plugin for LookTransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, look_transform_system);
    }
}

#[derive(Bundle, Clone)]
pub struct LookTransformBundle {
    pub transform: LookTransform,
    pub smoother: Smoother,
}

/// An eye and the target it's looking at. As a component, this can be modified in place of bevy's `Transform`, and the two will
/// stay in sync.
#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default, Debug, PartialEq)]
pub struct LookTransform {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

impl From<LookTransform> for Transform {
    fn from(t: LookTransform) -> Self {
        eye_look_at_target_transform(t.eye, t.target, t.up)
    }
}

impl Default for LookTransform {
    fn default() -> Self {
        Self {
            eye: Vec3::default(),
            target: Vec3::default(),
            up: Vec3::Y,
        }
    }
}

impl LookTransform {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        Self { eye, target, up }
    }

    pub fn radius(&self) -> f32 {
        (self.target - self.eye).length()
    }

    pub fn look_direction(&self) -> Option<Vec3> {
        (self.target - self.eye).try_normalize()
    }
}

fn eye_look_at_target_transform(eye: Vec3, target: Vec3, up: Vec3) -> Transform {
    // If eye and target are very close, we avoid imprecision issues by keeping the look vector a unit vector.
    let look_vector = (target - eye).normalize();
    let look_at = eye + look_vector;

    Transform::from_translation(eye).looking_at(look_at, up)
}

/// Preforms exponential smoothing on a `LookTransform`. Set the `lag_weight` between `0.0` and `1.0`, where higher is smoother.
#[derive(Clone, Component, Copy, Debug, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default, Debug)]
pub struct Smoother {
    lag_weight: f32,
    lerp_tfm: Option<LookTransform>,
    enabled: bool,
}

impl Default for Smoother {
    fn default() -> Self {
        Self {
            lag_weight: 0.9,
            lerp_tfm: Some(LookTransform::default()),
            enabled: true,
        }
    }
}

impl Smoother {
    pub fn new(lag_weight: f32) -> Self {
        Self {
            lag_weight,
            lerp_tfm: None,
            enabled: true,
        }
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if self.enabled {
            // To prevent camera jumping from last lerp before disabling to the current position,
            // reset smoother state
            self.reset();
        }
    }

    pub fn set_lag_weight(&mut self, lag_weight: f32) {
        self.lag_weight = lag_weight;
    }

    pub fn smooth_transform(&mut self, new_tfm: &LookTransform) -> LookTransform {
        debug_assert!(0.0 <= self.lag_weight);
        debug_assert!(self.lag_weight < 1.0);

        let old_lerp_tfm = self.lerp_tfm.unwrap_or(*new_tfm);

        let lead_weight = 1.0 - self.lag_weight;
        let lerp_tfm = LookTransform {
            eye: old_lerp_tfm.eye * self.lag_weight + new_tfm.eye * lead_weight,
            target: old_lerp_tfm.target * self.lag_weight + new_tfm.target * lead_weight,
            up: new_tfm.up,
        };

        self.lerp_tfm = Some(lerp_tfm);

        lerp_tfm
    }

    pub fn reset(&mut self) {
        self.lerp_tfm = None;
    }
}

pub fn look_transform_system(
    mut cameras: Query<(&LookTransform, &mut Transform, Option<&mut Smoother>)>,
) {
    for (look_transform, mut scene_transform, smoother) in cameras.iter_mut() {
        match smoother {
            Some(mut s) if s.enabled => {
                *scene_transform = s.smooth_transform(look_transform).into()
            }
            _ => (),
        };
    }
}
