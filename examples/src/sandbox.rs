use cgmath::Rotation3;
use gears_app::prelude::*;
use log::LevelFilter;
use std::f32::consts::PI;

#[tokio::main]
async fn main() -> EngineResult<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // Add FPS camera
    new_entity!(
        app,
        CameraMarker,
        Name("FPS Camera"),
        Pos3::new(cgmath::Vector3::new(30.0, 20.0, 30.0,)),
        ViewController::new_look_at(
            cgmath::Point3::new(30.0, 20.0, 30.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            0.8,
            0.0,
        ),
        MovementController::default(),
    );

    // Add ambient light
    new_entity!(
        app,
        LightMarker,
        Name("Ambient Light"),
        Pos3::new(cgmath::Vector3::new(0.0, 50.0, 0.0)),
        Light::Ambient { intensity: 0.05 },
    );

    // Add directional light
    new_entity!(
        app,
        LightMarker,
        Name("Directional Light"),
        Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,)),
        Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        },
    );

    // * Add moving red light
    let red_light = new_entity!(
        app,
        LightMarker,
        Name("Red Light"),
        Pos3::new(cgmath::Vector3::new(15.0, 5.0, 0.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [0.8, 0.0, 0.0],
            intensity: 1.0,
        },
    );

    // * Add moving blue light
    let blue_light = new_entity!(
        app,
        LightMarker,
        Name("Blue Light"),
        Pos3::new(cgmath::Vector3::new(-15.0, 5.0, 0.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 0.8],
            intensity: 1.0,
        },
    );

    // Red light
    new_entity!(
        app,
        LightMarker,
        Name("R"),
        Pos3::new(cgmath::Vector3::new(0.0, 5.0, -20.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [1.0, 0.0, 0.0],
            intensity: 1.0,
        },
    );

    // Green light
    new_entity!(
        app,
        LightMarker,
        Name("G"),
        Pos3::new(cgmath::Vector3::new(0.0, 5.0, -30.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [0.0, 1.0, 0.0],
            intensity: 1.0,
        },
    );

    // Blue light
    new_entity!(
        app,
        LightMarker,
        Name("B"),
        Pos3::new(cgmath::Vector3::new(0.0, 5.0, -40.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 1.0],
            intensity: 1.0,
        },
    );

    // * If you do not need the IDs of the entities you can chain them together
    app.new_entity() // Cube 1
        .add_component(Name("Cube1"))
        .add_component(ModelSource::Obj("models/cube/cube.obj"))
        .add_component(Pos3::new(cgmath::Vector3::new(10.0, 0.0, 10.0)))
        .new_entity() // Cube 2
        .add_component(Name("Cube2"))
        .add_component(ModelSource::Obj("models/cube/cube.obj"))
        .add_component(Pos3::new(cgmath::Vector3::new(10.0, 0.0, -10.0)))
        .new_entity() // Cube 3
        .add_component(Name("Cube3"))
        .add_component(ModelSource::Obj("models/cube/cube.obj"))
        .add_component(Pos3::new(cgmath::Vector3::new(-10.0, 0.0, -10.0)))
        .new_entity() // Cube 4
        .add_component(Name("Cube4"))
        .add_component(ModelSource::Obj("models/cube/cube.obj"))
        .add_component(Pos3::new(cgmath::Vector3::new(-10.0, 0.0, 10.0)))
        .build();

    // Center sphere
    new_entity!(
        app,
        StaticModelMarker,
        Name("Sphere1"),
        ModelSource::Obj("models/sphere/sphere.obj"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        Flip::Vertical
    );

    // Plane
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Plane"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-50.0, -0.1, -50.0),
            max: cgmath::Vector3::new(50.0, 0.1, 50.0),
        }),
        Pos3::new(cgmath::Vector3::new(0.0, -1.0, 0.0)),
        ModelSource::Obj("models/plane/plane.obj"),
    );

    // Add 5 spheres in a circle
    let mut moving_spheres: [Entity; 5] = [Entity::new(0); 5];
    for (i, sphere) in moving_spheres.iter_mut().enumerate() {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        let sphere_entity = new_entity!(
            app,
            Name(Box::leak(name.into_boxed_str())),
            ModelSource::Obj("models/sphere/sphere.obj"),
            Pos3::new(cgmath::Vector3::new(x, 0.0, z)),
        );

        *sphere = sphere_entity;
    }

    // Update loop

    async_system!(app, "update_sys", move |sa| {
        // ! Here we are inside a loop, so this has to lock on all iterations.
        let circle_speed = 8.0f32;
        let light_speed_multiplier = 3.0f32;

        // Move the spheres in a circle considering accumulated time
        for sphere in moving_spheres.iter() {
            let pos3 = sa.world.get_component::<Pos3>(*sphere).unwrap();

            let mut wlock_pos3 = pos3.write().unwrap();

            wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(PI * sa.dt.as_secs_f32() * circle_speed),
            ) * wlock_pos3.pos;
        }

        // Move the red and blue lights in a circle considering accumulated time
        if let Some(pos3) = sa.world.get_component::<Pos3>(red_light) {
            let mut wlock_pos3 = pos3.write().unwrap();

            wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(PI * sa.dt.as_secs_f32() * circle_speed * light_speed_multiplier),
            ) * wlock_pos3.pos;
        }

        if let Some(pos3) = sa.world.get_component::<Pos3>(blue_light) {
            let mut wlock_pos3 = pos3.write().unwrap();

            wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(PI * sa.dt.as_secs_f32() * circle_speed * light_speed_multiplier),
            ) * wlock_pos3.pos;
        }

        Ok(())
    });

    // Run the application
    app.run().await
}
