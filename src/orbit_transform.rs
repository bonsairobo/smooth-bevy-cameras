use bevy::{math::prelude::*, transform::components::Transform};

/// Two points, with one orbiting the other.
#[derive(Clone, Copy, Debug)]
pub struct OrbitTransform {
    pub pivot: Vec3,
    pub orbit: Vec3,
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

pub fn p1_look_at_p2_transform(p1: Vec3, p2: Vec3) -> Transform {
    // If p1 and p2 are very close, we avoid imprecision issues by keeping the look vector a unit
    // vector.
    let look_vector = (p2 - p1).normalize();
    let look_at = p1 + look_vector;

    Transform::from_translation(p1).looking_at(look_at, Vec3::Y)
}

#[derive(Default)]
pub struct Smoother {
    lerp_tfm: Option<OrbitTransform>,
}

impl Smoother {
    /// Do linear interpolation between the previous smoothed transform and the new transform. This is equivalent to an
    /// exponential smoothing filter.
    pub fn smooth_transform(
        &mut self,
        lag_weight: f32,
        new_tfm: &OrbitTransform,
    ) -> OrbitTransform {
        debug_assert!(0.0 <= lag_weight);
        debug_assert!(lag_weight < 1.0);

        let old_lerp_tfm = self.lerp_tfm.unwrap_or_else(|| *new_tfm);

        let lead_weight = 1.0 - lag_weight;
        let lerp_tfm = OrbitTransform {
            orbit: old_lerp_tfm.orbit * lag_weight + new_tfm.orbit * lead_weight,
            pivot: old_lerp_tfm.pivot * lag_weight + new_tfm.pivot * lead_weight,
        };

        self.lerp_tfm = Some(lerp_tfm);

        lerp_tfm
    }
}
