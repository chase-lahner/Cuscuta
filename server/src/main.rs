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
        server::send_player.after(server::listen),
        server::send_enemies,
    ))
    .run();
}