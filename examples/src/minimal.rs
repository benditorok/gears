use gears::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = GearsApp::default();

    app.new_entity() // Add fixed camera
        .add_component(components::Name("Fixed Camera"))
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            20.0, 15.0, 20.0,
        )))
        .add_component(components::Camera::Fixed {
            look_at: components::Pos3::new(cgmath::Vector3::new(0.0, 10.0, 0.0)),
        })
        .new_entity() // Add ambient light
        .add_component(components::Name("Ambient Light"))
        .add_component(components::Light::Ambient)
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 50.0, 0.0)))
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
