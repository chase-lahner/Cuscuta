use bevy::{prelude::*, time::common_conditions::on_timer, window::PresentMode};
use client::id_request;
use cuscuta_resources::TICKS_PER_SECOND;
use library::*;
use std::{env, time::Duration};
use markov_chains::*;
use player::CollisionState;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        /* room manager necessary? */
        .insert_resource(CollisionState::new())
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
            client::init_listen.after(id_request)),
        )
        .add_systems(Update, (
            player::move_player,
            client::listen,
            player::animate_player.after(player::move_player),
            enemies::handle_enemy_collision.after(player::move_player),
            player::player_attack.after(player::animate_player),
            player::player_roll.after(player::animate_player),
            camera::move_camera.after(player::animate_player),
            player::player_attack_enemy.after(player::player_attack),
            ui::update_ui_elements,
            player::player_interact,
            player::restore_health,
        )) 
        /* networking shtuff. comment out if needed */
        .add_systems(FixedUpdate,
            client::send_player
            //client::client_send_packets)
        )
        /* monkey stuff */
        .add_systems(Update, (
            player::spawn_monkey,
            player::update_monkey,
        ))  
        .run();
}
