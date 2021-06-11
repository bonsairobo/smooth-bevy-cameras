# smooth-bevy-cameras

A collection of exponentially-smoothed camera controllers for the Bevy Engine.

All controllers are based on a simple `OrbitTransform` component, which is just
a point that orbits around a pivot. By modifying this component, the scene graph
`Transform` will automatically be synchronized.

An `OrbitTransform` can easily be smoothed by adding a `Smoother` component.
