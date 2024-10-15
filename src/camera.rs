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
    ct.translation.y = pt.translation.y.clamp(-max_y + (WIN_H / 2.), max_y - (WIN_H / 2.) + (3. * (TILE_SIZE as f32)));
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
                height: Val::Px(96.),
                margin: UiRect{top: Val::VMin(0.), left: Val::VMax(0.),..default()},
                ..default()
            },
            ..default()
        },
        UiImage::new(asset_server.load("ui/ui_bar.png"))
    ));
    /* carnage bar spawn */
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(32.0),
                margin: UiRect{ top: Val::VMin(2.5), left: Val::VMax(37.5),..default()},
                ..default()
            },
            ..default()
        },
        UiImage::new(asset_server.load("ui/carnage_bar_border.png"))
    ));
}

