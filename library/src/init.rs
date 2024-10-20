use std::{net::{ Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket}, str::FromStr};

use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;

use crate::{camera::spawn_camera, carnage::*, cuscuta_resources, network::*, player::*, room_gen::*};


pub fn client_setup(
    mut commands: Commands, // to spawn in entities
    asset_server: Res<AssetServer>, // to access images
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>, // used in animation
    mut room_manager: ResMut<RoomManager>,
) {
    let ip_string = get_ip_addr();
    let mut sendable: Vec<u8> = Vec::new();

    // let socket = SocketAddrV4::new(Ipv4Addr::new(sendable[0], sendable[1],sendable[2],sendable[3]), split_u16);
    
    let mut addrs = ip_string.to_socket_addrs().unwrap();

    let socket_to_assign = addrs.next().unwrap();

    print!("Socket: {}",socket_to_assign);

    
    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server, &mut room_manager);

    /* initializes our networking socket */
    let socket = UdpSocket::bind("localhost:5000").unwrap(); //localhost:5000 THIS SHOULD BE REPLACED WITH CMD ARGS
    commands.insert_resource(UDP {socket: socket});
    

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
    let socket = UdpSocket::bind(cuscuta_resources::SERVER_ADR).unwrap();
    commands.insert_resource(UDP{socket:socket});
}
