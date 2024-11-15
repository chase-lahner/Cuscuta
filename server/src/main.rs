use cuscuta_resources::PlayerCount;
use library::*;
use bevy::prelude::*;

/* Rate at which we will be sending/recieving packets */
const TICKS_PER_SECOND: f64 = 60.;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, init::server_setup)
    .add_systems(FixedUpdate, (
        server::listen, 
        //player::update_player_position.after(server::listen),
        server::send_player
    ))
    .run();
}