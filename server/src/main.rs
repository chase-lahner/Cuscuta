use bevy::{prelude::*, time::common_conditions::on_timer};
use library::*;
use room_gen::RoomChangeEvent;
use std::{env, time::Duration};

/* Rate at which we will be sending/recieving packets */
const _TICKS_PER_SECOND: f64 = 60.;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        /* dont need no screen */
        .add_plugins(MinimalPlugins)
        /* for room change packet sending */
        .add_event::<RoomChangeEvent>()
        /* sets up server/start room */
        .add_systems(
            Startup,
            (
                init::server_setup,
                enemies::server_spawn_enemies.after(init::server_setup),
            ),
        )
        /* main logic, running at 60hz */
        .add_systems(
            FixedUpdate,
            (
                server::listen,

                server::check_door.after(server::listen),
                server::room_change_infodump.after(server::check_door),
                server::send_despawn_command,
                enemies::enemy_movement,
                server::send_enemies.after(server::listen),
                server::send_player.after(server::listen),
                server::send_despawn_command.after(server::listen),
                player::update_server_monkey,

            ),
        )
        .run();
}
