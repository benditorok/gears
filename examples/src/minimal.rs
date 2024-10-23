use gears::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = GearsApp::default();

    app.new_entity() // Add fixed camera
        .add_component(components::Name("Fixed Camera"))
        .add_component(components::Pos3::new(cgmath::Vector3::new(5.0, 5.0, 5.0)))
        .add_component(components::Camera::Fixed {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
        })
        .build();

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

    let _sphere = app
        .new_entity() // Add a sphere
        .add_component(components::Name("Sphere1"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)))
        .build();

    app.run().await?;
    Ok(())
}
