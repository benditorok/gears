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
- [x] Player
- [x] Movement and View controllers
- [x] Rigidbody physics
- [ ] Gltf animation handling
- [ ] Anti aliasing
- [ ] HDR textures

![Demo](/doc/imgs/demo4.png)

## Examples

You can try it with `cargo run --bin minimal` or run a more complex example with `cargo run --bin sandbox`.
When creating components you can use a macro or an entity builder as well.

### Creating entities

```rust
    let mut app = GearsApp::default();

    let red_light = new_entity!(
        app,
        LightMarker,
        components::Pos3::new(cgmath::Vector3::new(15.0, 5.0, 0.0))
        components::Name("Red Light"),
        components::Light::PointColoured {
            radius: 10.0,
            color: [0.8, 0.0, 0.0],
            intensity: 1.0,
        },
    );
```

### Add a custom window

```rust
   app.add_window(Box::new(move |ui| {
        egui::Window::new("Window")
            .default_open(true)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            .default_pos([0.5, 0.5])
            .show(ui, |ui| {
                if ui.add(egui::Button::new("Click me")).clicked() {
                    warn!("Button clicked in the custom window!");
                }
                ui.end_row();
            });
    }));
```

### Update entities

```rust
 app.update_loop(move |world, dt| {
        let spin_speed = 0.5f32;

        if let Some(pos3) = world.get_component::<Pos3>(sphere_entity) {
            let mut wlock_pos3 = pos3.write().unwrap();

            let rotation = wlock_pos3.rot;
            wlock_pos3.rot =
                Quaternion::from_angle_y(cgmath::Rad(dt.as_secs_f32() * spin_speed)) * rotation;
        }
    })
    .await?;
```
