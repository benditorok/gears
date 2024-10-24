use cgmath::{One, Quaternion, Rotation3};
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
    let sphere_entity = new_entity!(
        app,
        components::Name("Sphere1"),
        components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj",
        },
        components::Pos3::with_rot(cgmath::Vector3::new(0.0, 0.0, 0.0), Quaternion::one()),
    );

    // Use the update loop to spin the sphere
    app.update_loop(move |ecs, dt| {
        let ecs = ecs.lock().unwrap();
        let spin_speed = 0.5f32;

        if let Some(pos) = ecs.get_component_from_entity::<components::Pos3>(sphere_entity) {
            let mut pos3 = pos.write().unwrap();

            pos3.rot = Some(
                Quaternion::from_angle_y(cgmath::Rad(dt.as_secs_f32() * spin_speed))
                    * pos3.rot.unwrap(),
            );
        }
    })
    .await?;

    app.run().await
}
