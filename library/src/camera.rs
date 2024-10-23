use bevy::prelude::*;

use crate::{cuscuta_resources::*, room_gen::*, player::*};


pub fn move_camera(
    player: Query<(&Transform,&NetworkId), With<Player>>,
    mut camera: Query<&mut Transform, (Without<Player>, With<Camera>)>,
    room_manager: Res<RoomManager>, // Access the RoomManager to get the room-specific max_x and max_y
    client_id: Res<ClientId>
) {
    for (transform, id) in player.iter()
    {
        if id.id == client_id.id{
            let mut ct = camera.single_mut();
            // Retrieve the dynamically calculated max_x and max_y from RoomManager
            let (max_x, max_y) = room_manager.current_room_max();


        // Retrieve the dynamically calculated max_x and max_y from RoomManager
        // let (max_x, max_y) = room_manager.current_room_max();
        // println!("1. MAX X: {} MAX Y: {}",max_x,max_y);

        let (width, height) = room_manager.current_room_size();
        //println!("2. WIDTH: {} HEIGHT: {}",width,height);

        let max_x = width/2.0;
        let max_y = height/2.0;

        ct.translation.x = transform.translation.x.clamp(-max_x + (WIN_W / 2.), max_x - (WIN_W / 2.));
        ct.translation.y = transform.translation.y.clamp(-max_y + (WIN_H / 2.), max_y - (WIN_H / 2.) + (3. * (TILE_SIZE as f32)));
        }
    }
}

pub fn spawn_camera(
    commands: &mut Commands,
    asset_server: & AssetServer
){
    /* camera spawn */
    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));
    
    /* ui bar */
    commands.spawn((
        NodeBundle {
            style: Style{
                width: Val::Px(1280.),
                /* currently a 3 tile tall bar is plopped at top of screen */
                height: Val::Px(96.),
                margin: UiRect{top: Val::VMin(0.), left: Val::VMax(0.),..default()},
                ..default()
            },
            z_index: ZIndex::Global(0),
            ..default()
        },
        UiImage::new(asset_server.load("ui/ui_bar.png"))
    ));
}

