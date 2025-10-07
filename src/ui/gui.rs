//! Components for the graphical user interface.

use bevy::prelude::*;

pub fn make_gui() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup);
    app.add_systems(Update, toggle_wireframe);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let rect = meshes.add(Rectangle::new(50.0, 50.0));
    let circle = meshes.add(Circle::new(50.0));
    let capsule = meshes.add(Capsule2d::new(25.0, 50.0));

    let n_pits = 6usize;
    let n_pits_f = n_pits as f32;

    for i in 0..n_pits {
        commands.spawn((
            Mesh2d(circle.clone()),
            Transform::from_xyz(i as f32 * 100. - 2. * 100. - 50., 0.0, 0.0),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 1.0))),
        ));
    }

    commands.spawn((
        Mesh2d(rect.clone()),
        Transform::from_xyz(0.0, 0.0, 1.0),
        MeshMaterial2d(materials.add(Color::srgb(1., 0.25, 0.25))),
    ));

    commands.spawn((
        Text::new("Press space to toggle wireframes"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn toggle_wireframe(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
       println!("toggle_wireframe");
    }
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(position) = window.cursor_position() {
            println!("Cursor is inside the primary window, at {:?}", position);
        } else {
            println!("Cursor is not in the game window.");
        }
    }
}
