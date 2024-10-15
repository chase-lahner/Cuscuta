use bevy::prelude::*;

use crate::{cuscuta_resources::*, room_gen::*};


pub fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (Without<Player>, With<Camera>)>,
    room_manager: Res<RoomManager>, // Access the RoomManager to get the room-specific max_x and max_y
) {
    let pt = player.single();
    let mut ct = camera.single_mut();

    // Retrieve the dynamically calculated max_x and max_y from RoomManager
    let (max_x, max_y) = room_manager.current_room_max();

    ct.translation.x = pt.translation.x.clamp(-max_x + (WIN_W / 2.), max_x - (WIN_W / 2.));
    ct.translation.y = pt.translation.y.clamp(-max_y + (WIN_H / 2.), max_y - (WIN_H / 2.));
}

