use bevy::{prelude::*, window::PresentMode};
use library::*;

fn main() {
    App::new()
        .insert_resource(room_gen::RoomManager::new())
        .add_systems(PreStartup, init::ip_setup) // should run before we spawn / send data to server
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // need window!
                title: cuscuta_resources::TITLE.into(),
                present_mode: PresentMode::Fifo,
                ..default() // Name and present mode all we need for now
             }),
             ..default()
         }))
         .add_systems(Startup,(
            init::client_setup, 
            client::id_request.after(init::client_setup)
        ))
         .add_systems(Startup, enemies::spawn_enemies)
         .add_systems(Update, player::move_player)// every frame, takes in WASD for movement
         //.add_systems(Update, (
            // player::player_input, 
            // player::update_player_position.after(player::player_input),
            // client::send_player.after(player::update_player_position)))
        .add_systems(Update, client::listen)
        .add_systems(Update, client::send_player.after(client::listen))
        .add_systems(Update, enemies::enemy_movement.after(player::move_player))
        .add_systems(Update, player::animate_player.after(player::move_player)) // animates player
        .add_systems(Update, player::player_attack.after(player::animate_player)) // animates attack swing
        .add_systems(Update, player::player_roll.after(player::animate_player)) // animates roll
        .add_systems(Update, camera::move_camera.after(player::animate_player)) // follow character
        .add_systems(Update, player::player_attack_enemy.after(player::player_attack)) // kills enemies
        .add_systems(Update, player::player_interact)
        .run();
}
