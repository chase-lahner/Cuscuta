use bevy::{prelude::*, time::common_conditions::on_timer};
use library::*;
use std::{env, time::Duration};

/* Rate at which we will be sending/recieving packets */
const _TICKS_PER_SECOND: f64 = 60.;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(
            Startup,
            (
                init::server_setup,
                enemies::server_spawn_enemies.after(init::server_setup),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                server::listen,//.run_if(on_timer(Duration::from_millis(5))),
                enemies::enemy_movement.after(server::listen), // server needs to handle this :3
                server::send_enemies.after(server::listen),
                server::send_player.after(server::listen),
                //server::server_send_packets.after(server::send_player),
            ),
        )
        .run();
}
