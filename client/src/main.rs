use bevy::{prelude::*, window::PresentMode};
use cuscuta_resources::TICKS_PER_SECOND;
use library::*;

fn main() {
    App::new()
        .insert_resource(room_gen::RoomManager::new())
        .insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND))
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
            //player::player_input,
            //player::update_player_position.after(player::player_input),
            enemies::enemy_movement.after(player::move_player),
            player::animate_player.after(player::move_player),
            player::player_attack.after(player::animate_player),
            player::player_roll.after(player::animate_player),
            camera::move_camera.after(player::animate_player),
            player::player_attack_enemy.after(player::player_attack),
            ui::update_ui_elements,
            player::player_interact
        )) 
        /* networking shtuff. comment out if needed */
        .add_systems(FixedUpdate,
            (client::send_player,
            client::listen
        ))
        .run();
}
