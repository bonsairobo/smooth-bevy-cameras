# smooth-bevy-cameras

A collection of exponentially-smoothed camera controllers for the Bevy Engine.

All controllers are based on a simple `OrbitTransform` component, which is just
a point that orbits around a pivot. By modifying this component, the scene graph
`Transform` will automatically be synchronized.

An `OrbitTransform` can easily be smoothed by adding a `Smoother` component.

```rust
// Enables the system that synchronizes your `Transform`s and `OrbitTransform`s.
app.add_plugin(OrbitTransformPlugin);

...

// A 3rd person camera.
commands
    .spawn_bundle(OrbitTransformBundle {
        transform: OrbitTransform { orbit: eye, pivot: eye + look_direction },
        polarity: LookPolarity::OrbitLookAtPivot,
        smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
    })

...

// A 1st person camera.
commands
    .spawn_bundle(OrbitTransformBundle {
        transform: OrbitTransform { orbit: eye + look_direction, pivot: eye },
        polarity: LookPolarity::PivotLookAtOrbit,
        smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
    })
```

## Built-In Controllers

- `UnrealCameraBundle + UnrealCameraPlugin`
- `OrbitCameraBundle + OrbitCameraPlugin`
