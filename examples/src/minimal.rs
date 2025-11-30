use cgmath::{Quaternion, Rotation3};
use gears_app::prelude::*;
use log::LevelFilter;

#[tokio::main]
async fn main() -> EngineResult<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // Add fixed camera
    new_entity!(
        app,
        CameraMarker,
        Name("Fixed Camera"),
        Pos3::new(cgmath::Vector3::new(3.0, 2.0, 3.0)),
        ViewController::new_look_at(
            cgmath::Point3::new(3.0, 2.0, 3.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            0.0,
            0.0
        )
    );

    // Add ambient light
    new_entity!(
        app,
        LightMarker,
        Name("Ambient Light"),
        Light::Ambient { intensity: 0.1 },
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        LightMarker,
        Name("Directional Light"),
        Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.6,
        },
        Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // Add a green light
    new_entity!(
        app,
        LightMarker,
        Name("Green Light"),
        Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.8, 0.0],
            intensity: 0.6,
        },
        Pos3::new(cgmath::Vector3::new(-4.0, 4.0, 4.0)),
    );

    // Add a sphere and get the Entity for reference
    let sphere_entity = new_entity!(
        app,
        StaticModelMarker,
        Name("Sphere1"),
        ModelSource::Obj("models/sphere/sphere.obj"),
        Pos3::default(),
    );

    // Create a system to rotate the sphere
    async_system!(app, "update_rot", move |world, dt| {
        const SPIN_SPEED: f32 = 1_f32;
        if let Some(pos3) = world.get_component::<Pos3>(sphere_entity) {
            let mut wlock_pos3 = pos3.write().unwrap();

            let rotation = wlock_pos3.rot;
            wlock_pos3.rot = Quaternion::from_angle_y(cgmath::Rad(
                SPIN_SPEED * dt.as_secs_f32(), // Scale by delta time
            )) * rotation;
        }

        Ok(())
    });

    // Run the application
    app.run()
}
