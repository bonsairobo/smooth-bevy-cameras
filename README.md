# smooth-bevy-cameras

A collection of exponentially-smoothed camera controllers for the Bevy Engine.

All controllers are based on a `LookTransform` component, which is just an `eye` point that looks at a `target` point. By
modifying this component, the scene graph `Transform` will automatically be synchronized.

A `LookTransform` can be smoothed by adding a `Smoother` component, and the smoothing will happen automatically.

```rust
// Enables the system that synchronizes your `Transform`s and `LookTransform`s.
app.add_plugin(LookTransformPlugin);

...

commands
    .spawn_bundle(LookTransformBundle {
        transform: LookTransform { eye, target },
        smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
    })
```

## Look Angles

When implementing a camera controller, it's often useful to work directly with the angles (pitch and yaw) of your look
direction. You can do this with the `LookAngles` type:

```rust
let mut angles = LookAngles::from_vector(transform.look_direction());
angles.add_pitch(0.1);
angles.add_yaw(0.1);
transform.offset_target_in_direction(angles.unit_vector());
```

This is how the built-in controllers implement rotation controls.

## Built-In Controllers

These plugins depend on the `LookTransformPlugin`:

- `UnrealCameraPlugin + UnrealCameraBundle`
- `OrbitCameraPlugin + OrbitCameraBundle`
