use std::net::UdpSocket;

use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;

use crate::{camera::spawn_camera, carnage::*, cuscuta_resources::{self, FlexSerializer}, network::*, player::*, room_gen::*};


pub fn client_setup(
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
    commands.insert_resource(FlexSerializer{serializer:flexbuffers::FlexbufferSerializer::new()});
    

    // spawn camera
    spawn_camera(&mut commands, &asset_server);

    client_spawn_carnage_bar(&mut commands, &asset_server);
    /* spawn pot to play with */
    client_spawn_pot(&mut commands, &asset_server);
    // spawn player
    client_spawn_player(&mut commands, &asset_server, &mut texture_atlases);
}


pub fn server_setup(
    mut commands: Commands
){
    info!("entered setup");
    let socket = UdpSocket::bind(cuscuta_resources::SERVER_ADR).unwrap();
    commands.insert_resource(UDP{socket:socket});
    commands.insert_resource(FlexSerializer{serializer:flexbuffers::FlexbufferSerializer::new()});
    info!("done setup");
}
