# Gears

A 3D game engine written in Rust using wgpu for rendering.

Goals

- Ease of use
- Cross platform compatibility
- Parallel execution where possible

## Current progress

- [x] Load 3D objects
- [x] Generic lights
- [ ] Shadows

![Demo](/doc/imgs/demo3.png)

## Simple example

You can try it with `cargo run --bin minimal` or run a more complex example with `cargo run --bin sandbox`.
When creating components you can use a macro or an entity builder as well.

```rust
use gears::{new_entity, prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = GearsApp::default();

    // Add fixed camera
    new_entity!(
        app,
        components::Name("Fixed Camera"),
        components::Pos3::new(cgmath::Vector3::new(5.0, 5.0, 5.0)),
        components::Camera::Fixed {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    );

    // Use the entity builder
    app.new_entity() // Add ambient light
        .add_component(components::Name("Ambient Light"))
        .add_component(components::Light::Ambient { intensity: 0.05 })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 50.0, 0.0)))
        .new_entity() // Add directional light
        .add_component(components::Name("Directional Light"))
        .add_component(components::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            30.0, 30.0, 30.0,
        )))
        .new_entity() // Add a green light
        .add_component(components::Name("Green Light"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.8, 0.0],
            intensity: 0.6,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(-4.0, 4.0, 4.0)))
        .build();

    // Add a sphere and get the Entity for reference
    let _sphere_entity = new_entity!(
        app,
        components::Name("Sphere1"),
        components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj",
        },
        components::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
    );

    app.run().await?;
    Ok(())
}
```
