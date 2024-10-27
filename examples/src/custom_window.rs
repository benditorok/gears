use cgmath::{One, Quaternion, Rotation3};
use gears::{new_entity, prelude::*};
use log::{warn, LevelFilter};

pub fn example_gui(ui: &egui::Context) {
    egui::Window::new("TestWindow")
        .default_open(true)
        .max_width(1000.0)
        .max_height(800.0)
        .default_width(800.0)
        .resizable(true)
        .default_pos([0.0, 0.0])
        .show(ui, |ui| {
            if ui.add(egui::Button::new("Click me")).clicked() {
                warn!("Button clicked in the custom window!");
            }
            ui.end_row();
        });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // Add the custom window
    app.add_window(Box::new(example_gui));

    app.add_window(Box::new(move |ui| {
        egui::Window::new("Window")
            .default_open(true)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            .default_pos([400.0, 0.0])
            .show(ui, |ui| {
                if ui.add(egui::Button::new("Click me")).clicked() {
                    warn!("Button clicked in the custom window!");
                }
                ui.end_row();
            });
    }));

    // Add fixed camera
    new_entity!(
        app,
        components::Name("Fixed Camera"),
        components::transform::Pos3::new(cgmath::Vector3::new(3.0, 2.0, 3.0)),
        components::Camera::Fixed {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
        }
    );

    // Use the entity builder
    app.new_entity() // Add ambient light
        .add_component(components::Name("Ambient Light"))
        .add_component(components::light::Light::Ambient { intensity: 0.05 })
        .add_component(components::transform::Pos3::new(cgmath::Vector3::new(
            0.0, 50.0, 0.0,
        )))
        .new_entity() // Add directional light
        .add_component(components::Name("Directional Light"))
        .add_component(components::light::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        })
        .add_component(components::transform::Pos3::new(cgmath::Vector3::new(
            30.0, 30.0, 30.0,
        )))
        .new_entity() // Add a green light
        .add_component(components::Name("Green Light"))
        .add_component(components::light::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.8, 0.0],
            intensity: 0.6,
        })
        .add_component(components::transform::Pos3::new(cgmath::Vector3::new(
            -4.0, 4.0, 4.0,
        )))
        .build();

    // Add a sphere and get the Entity for reference
    let sphere_entity = new_entity!(
        app,
        components::Name("Sphere1"),
        components::model::ModelSource {
            obj_path: "res/models/sphere/sphere.obj",
        },
        components::model::StaticModel {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
        },
    );

    // Use the update loop to spin the sphere
    app.update_loop(move |ecs, dt| {
        let ecs = ecs.lock().unwrap();
        let spin_speed = 0.5f32;

        if let Some(static_model) =
            ecs.get_component_from_entity::<components::model::StaticModel>(sphere_entity)
        {
            let mut wlock_static_model = static_model.write().unwrap();

            let rotation = wlock_static_model.rotation;
            wlock_static_model.rotation =
                Quaternion::from_angle_y(cgmath::Rad(dt.as_secs_f32() * spin_speed)) * rotation;
        }
    })
    .await?;

    app.run().await
}
