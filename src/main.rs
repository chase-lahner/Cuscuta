use bevy::{prelude::*, window::PresentMode};

#[derive(Component, Deref, DerefMut)]
struct PopupTimer(Timer);

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