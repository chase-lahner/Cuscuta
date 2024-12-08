use bevy::prelude::*;

use crate::{cuscuta_resources::*, room_gen::*, player::*};


/* adjusts camera, clamped to our room */
pub fn move_camera(
    player: Query<(&Transform,&NetworkId), With<Player>>,
    mut camera: Query<&mut Transform, (Without<Player>, With<Camera>)>,
    room_manager: Res<ClientRoomManager>, // Access the RoomManager to get the room-specific max_x and max_y
    client_id: Res<ClientId>
) {
    /* iterate players, find us */
    for (transform, id) in player.iter()
    {
        /* are we us? */
        if id.id == client_id.id{
            /* just one cam */
            let mut ct = camera.single_mut();

            /* width in pixels */
            let width = room_manager.width;
            let height = room_manager.height;

            /* (max,max) for each quadrant (big rectangle) */
            let max_x = width/2.0;
            let max_y = height/2.0;

            
        ct.translation.x = transform.translation.x.clamp(-max_x + (WIN_W / 2.), max_x - (WIN_W / 2.));
        ct.translation.y = transform.translation.y.clamp(-max_y + (WIN_H / 2.), max_y - (WIN_H / 2.));
        }
    }
}

pub fn spawn_camera(
    commands: &mut Commands,
    // asset_server: & AssetServer
){
    /* camera spawn */
    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));

}

