use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    math::prelude::*,
    transform::components::Transform,
};

pub struct OrbitTransformPlugin;

impl Plugin for OrbitTransformPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(orbit_transform_system.system());
    }
}

#[derive(Bundle)]
pub struct OrbitTransformBundle {
    pub transform: OrbitTransform,
    pub polarity: LookPolarity,
    pub smoother: Smoother,
}

/// Two points, with one orbiting the other. As a component, this can be modified in place of bevy's `Transform`, and the two
/// will stay in sync.
#[derive(Clone, Copy, Debug)]
pub struct OrbitTransform {
    pub pivot: Vec3,
    pub orbit: Vec3,
}

/// Which way an `OrbitTransform` looks.
#[derive(Clone, Copy, Debug)]
pub enum LookPolarity {
    /// The camera faces from the pivot to the orbit.
    PivotLookAtOrbit,
    /// The camera faces from the orbit to the pivot.
    OrbitLookAtPivot,
}

impl OrbitTransform {
    pub fn radius(&self) -> f32 {
        (self.pivot - self.orbit).length()
    }

    pub fn pivot_look_at_orbit_transform(&self) -> Transform {
        p1_look_at_p2_transform(self.pivot, self.orbit)
    }

    pub fn orbit_look_at_pivot_transform(&self) -> Transform {
        p1_look_at_p2_transform(self.orbit, self.pivot)
    }

    pub fn look_at(&self, polarity: LookPolarity) -> Transform {
        match polarity {
            LookPolarity::OrbitLookAtPivot => self.orbit_look_at_pivot_transform(),
            LookPolarity::PivotLookAtOrbit => self.pivot_look_at_orbit_transform(),
        }
    }

    pub fn orbit_to_pivot_direction(&self) -> Vec3 {
        (self.pivot - self.orbit).normalize()
    }

    pub fn pivot_to_orbit_direction(&self) -> Vec3 {
        (self.orbit - self.pivot).normalize()
    }

    pub fn set_orbit_in_direction(&mut self, direction: Vec3) {
        self.orbit = self.pivot + self.radius() * direction;
    }
}

fn p1_look_at_p2_transform(p1: Vec3, p2: Vec3) -> Transform {
    // If p1 and p2 are very close, we avoid imprecision issues by keeping the look vector a unit vector.
    let look_vector = (p2 - p1).normalize();
    let look_at = p1 + look_vector;

    Transform::from_translation(p1).looking_at(look_at, Vec3::Y)
}

pub struct Smoother {
    lag_weight: f32,
    lerp_tfm: Option<OrbitTransform>,
}

impl Smoother {
    pub fn new(lag_weight: f32) -> Self {
        Self {
            lag_weight,
            lerp_tfm: None,
        }
    }

    pub fn set_lag_weight(&mut self, lag_weight: f32) {
        self.lag_weight = lag_weight;
    }

    /// Do linear interpolation between the previous smoothed transform and the new transform. This is equivalent to an
    /// exponential smoothing filter.
    pub fn smooth_transform(&mut self, new_tfm: &OrbitTransform) -> OrbitTransform {
        debug_assert!(0.0 <= self.lag_weight);
        debug_assert!(self.lag_weight < 1.0);

        let old_lerp_tfm = self.lerp_tfm.unwrap_or_else(|| *new_tfm);

        let lead_weight = 1.0 - self.lag_weight;
        let lerp_tfm = OrbitTransform {
            orbit: old_lerp_tfm.orbit * self.lag_weight + new_tfm.orbit * lead_weight,
            pivot: old_lerp_tfm.pivot * self.lag_weight + new_tfm.pivot * lead_weight,
        };

        self.lerp_tfm = Some(lerp_tfm);

        lerp_tfm
    }
}

pub fn orbit_transform_system(
    mut cameras: Query<(
        &OrbitTransform,
        &mut Transform,
        Option<&LookPolarity>,
        Option<&mut Smoother>,
    )>,
) {
    for (orbit_tfm, mut scene_transform, polarity, smoother) in cameras.iter_mut() {
        let effective_orbit_tfm = if let Some(mut smoother) = smoother {
            smoother.smooth_transform(orbit_tfm)
        } else {
            orbit_tfm.clone()
        };

        let polarity = polarity.cloned().unwrap_or(LookPolarity::OrbitLookAtPivot);
        *scene_transform = effective_orbit_tfm.look_at(polarity);
    }
}
