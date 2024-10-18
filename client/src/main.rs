use bevy::{prelude::*, window::PresentMode};
use library::*;

fn main() {
    App::new()
        .insert_resource(room_gen::RoomManager::new())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // need window!
                title: cuscuta_resources::TITLE.into(),
                present_mode: PresentMode::Fifo,
                ..default() // Name and present mode all we need for now
             }),
             ..default()
         }))
         .add_systems(Startup,init::client_setup)// runs once, sets up scene
         .add_systems(Startup, enemies::spawn_enemies)
         .add_systems(Update, player::move_player)// every frame, takes in WASD for movement
         .add_systems(Startup, network::client_send_id_packet.after(init::client_setup)) // we want id when we spawn a player
        // .add_systems(Update, network::recv_packet)
        .add_systems(Startup, network::recv_id.after(network::send_id_packet)) // we want to recieve packet after we send it
        .add_systems(
            Update,
            network::send_movement_info.after(player::move_player),
        )
        .add_systems(Update, network::serialize_player.after(player::move_player))
        .add_systems(Update, enemies::enemy_movement.after(player::move_player))
        .add_systems(Update, player::animate_player.after(player::move_player)) // animates player
        .add_systems(Update, player::player_attack.after(player::animate_player)) // animates attack swing
        .add_systems(Update, camera::move_camera.after(player::animate_player)) // follow character
        .add_systems(Update, player::player_interact)
        .run();
}
