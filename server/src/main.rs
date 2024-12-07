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
                server::listen,
                server::send_despawn_command,
                enemies::enemy_movement,
                server::send_enemies.after(server::listen),
                server::send_player.after(server::listen),
                server::send_despawn_command.after(server::listen),
            ),
        )
        .run();
}
