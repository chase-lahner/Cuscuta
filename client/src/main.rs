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
            client::id_request.after(init::client_setup),
            enemies::spawn_enemies)
        )
        .add_systems(Update, (
            player::move_player,
            player::player_input,
            player::update_player_position.after(player::player_input),
            client::send_player.after(player::update_player_position),
            client::listen,
            client::send_player.after(client::listen),
            enemies::enemy_movement.after(player::move_player),
            player::animate_player.after(player::move_player),
            player::player_attack.after(player::animate_player),
            player::player_roll.after(player::animate_player),
            camera::move_camera.after(player::animate_player),
            player::player_attack_enemy.after(player::player_attack)
            //player::player_interact,
        )) 
        //  player::player_interact)
        .run();
}
