use bevy::prelude::*;
use hexlands_core::types::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Simple camera
    commands.spawn(Camera2dBundle::default());
    // spawn a simple UI text indicating the app is running
    commands.spawn(TextBundle::from_section("Hexlands App", TextStyle {
        font_size: 40.0,
        color: Color::WHITE,
        ..Default::default()
    }));
}
