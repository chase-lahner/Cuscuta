use bevy::{prelude::*, window::PresentMode};
// dependencies

#[derive(Component, Deref, DerefMut)]
struct PopupTimer(Timer);
// Timers

// Little hello_world credit scene.
// Uses Sprite Bundle with each image spawned in a stack on z axis
// Timer waits 3s, then transforms next image

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {// need window!
               title: "Credit Sequence!".into(),
               present_mode: PresentMode::Fifo,
               ..default() // Name and present mode all we need for now
            }),
            ..default()
        }))
        .add_systems(Startup,setup)
        .add_systems(Update, show_popup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>){
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {// first image to show
        texture: asset_server.load("")
        ..default()
    });
    commands
        .spawn(SpriteBundle {// second image
            texture: asset_server.load(""),
            transform: Transform::from_xyz(0.,0.,-1.),
            ..default()
        })
        .insert(PopupTime(Timer::from_seconds(3.,TimerMode::Once)));
    commands
        .spawn(SpriteBundle {// third image
            texture: asset_server.load(""),
            transform: Transform::from_xyz(0.,0.,-2.),
            ..default()
        })
        .insert(PopupTime(Timer::from_seconds(6.,TimerMode::Once)));
    commands
        .spawn(SpriteBundle {// fourth image
            texture: asset_server.load(""),
            transform: Transform::from_xyz(0.,0.,-3.),
            ..default()
        })
        .insert(PopupTime(Timer::from_seconds(9.,TimerMode::Once)));
    commands
        .spawn(SpriteBundle { // fifth Image
            texture: asset_server.load(""),
            transform: Transform::from_xyz(0.,0.,-4.),
            ..default()
        })
        .insert(PopupTime(Timer::from_seconds(12.,TimerMode::Once)));
    info!("Hello World!");
}

fn show_popup(time: Res<Time>, mut popup: Query<(&mut PopupTimer, &mut Transform)>){
    for (mut timer, mut transform) in popup.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished(){
            transform.translation *= -2;// stacked increasingly in -z, this pulls em back!
            info!("Swapped pics!");
        }

    }
}
