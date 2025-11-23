use cgmath::{Rotation3, Vector3};
use gears_app::prelude::*;

pub fn setup_map(app: &mut GearsApp) {
    // Create ground plane
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Ground Plane"),
        RigidBody::new_static(AABBCollisionBox {
            min: Vector3::new(-100.0, -0.1, -100.0),
            max: Vector3::new(100.0, 0.1, 100.0),
        }),
        Pos3::new(Vector3::new(0.0, -1.0, 0.0)),
        ModelSource::Obj("models/plane/plane.obj"),
    );

    // Wall 1
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 1"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new(Vector3::new(-51.0, 1.0, 0.0)),
        ModelSource::Obj("models/wall_50/wall_50.obj")
    );

    // Wall 2
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 2"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(0.0, 1.0, -51.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0)), // Rotate wall 90 degrees around Y-axis
        ),
        ModelSource::Obj("models/wall_50/wall_50.obj")
    );

    // Wall 3
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 3"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new(Vector3::new(51.0, 1.0, 0.0)),
        ModelSource::Obj("models/wall_50/wall_50.obj")
    );

    // Wall 4
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 4"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(0.0, 1.0, 51.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0)), // Rotate wall 90 degrees around Y-axis
        ),
        ModelSource::Obj("models/wall_50/wall_50.obj")
    );

    // // Outer boundary walls - North wall
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 0"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-45.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 1"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-35.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 2"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-25.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 3"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-15.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 4"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-5.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 5"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(5.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 6"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(15.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 7"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(25.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 8"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(35.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("North Wall 9"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(45.0, 1.0, -45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );

    // // South wall
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 0"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-45.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 1"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-35.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 2"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-25.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 3"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-15.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 4"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(-5.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 5"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(5.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 6"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(15.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 7"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(25.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 8"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(35.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("South Wall 9"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new_with_rot(
    //         Vector3::new(45.0, 1.0, 45.0),
    //         Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
    //     ),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );

    // // West wall
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 0"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, -45.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 1"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, -35.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 2"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, -25.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 3"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, -15.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 4"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, -5.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 5"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, 5.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 6"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, 15.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 7"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, 25.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 8"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, 35.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("West Wall 9"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(-45.0, 1.0, 45.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );

    // // East wall
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 0"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, -45.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 1"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, -35.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 2"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, -25.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 3"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, -15.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 4"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, -5.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 5"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, 5.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 6"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, 15.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 7"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, 25.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 8"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, 35.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     ObstacleMarker,
    //     Name("East Wall 9"),
    //     RigidBody::new_static(AABBCollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
    //         max: cgmath::Vector3::new(1.0, 2.0, 10.0),
    //     }),
    //     Pos3::new(Vector3::new(45.0, 1.0, 45.0)),
    //     ModelSource::Obj("models/wall_10/wall_10.obj")
    // );

    // Interior labyrinth walls - Vertical walls
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V1"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(-25.0, 1.0, -30.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V2"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(-25.0, 1.0, -10.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V3"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(-5.0, 1.0, -30.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V4"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(-5.0, 1.0, 10.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V5"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(15.0, 1.0, -30.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V6"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(15.0, 1.0, 0.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V7"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(35.0, 1.0, -20.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze V8"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new(Vector3::new(35.0, 1.0, 20.0)),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );

    // Horizontal walls (rotated 90 degrees)
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H1"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(-35.0, 1.0, -25.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H2"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(-15.0, 1.0, -25.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H3"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(-35.0, 1.0, 5.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H4"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(-15.0, 1.0, 5.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H5"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(5.0, 1.0, -15.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H6"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(25.0, 1.0, -15.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H7"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(5.0, 1.0, 25.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H8"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(25.0, 1.0, 25.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H9"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(-35.0, 1.0, 35.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Maze H10"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -10.0),
            max: cgmath::Vector3::new(1.0, 2.0, 10.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(-5.0, 1.0, 35.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0))
        ),
        ModelSource::Obj("models/wall_10/wall_10.obj")
    );
}
