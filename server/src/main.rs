use bevy::prelude::*;
use library::*;
use std::env;

/* Rate at which we will be sending/recieving packets */
const TICKS_PER_SECOND: f64 = 60.;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (
                init::server_setup,
                enemies::server_spawn_enemies.after(init::server_setup),
            ),
        )
        .add_systems(Update,
        server::listen)
        .add_systems(
            FixedUpdate,
            (
                server::send_player.after(server::send_enemies),
                server::send_enemies,
                server::server_send_packets.after(server::send_player),
            ),
        )
        .run();
}
