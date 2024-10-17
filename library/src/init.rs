use std::net::UdpSocket;

use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;

use crate::{camera::spawn_camera, carnage::*, network::*, player::*, room_gen::*};


pub fn setup(
    mut commands: Commands, // to spawn in entities
    asset_server: Res<AssetServer>, // to access images
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>, // used in animation
    mut room_manager: ResMut<RoomManager>,
) {
    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server, &mut room_manager);

    /* initializes our networking socket */
    let socket = UdpSocket::bind("localhost:5000").unwrap();
    commands.insert_resource(UDP {socket: socket});

    commands.insert_resource(Attacking{attack: false});

    

    // spawn camera
    spawn_camera(&mut commands, &asset_server);

    spawn_carnage_bar(&mut commands, &asset_server);
    /* spawn pot to play with */
    spawn_pot(&mut commands, &asset_server);
    // spawn player
    spawn_player(&mut commands, &asset_server, &mut texture_atlases);
}

