use std::net::UdpSocket;

use bevy::prelude::*;

use crate::{cuscuta_resources::*, network::*, player::*, room_gen::*};


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
    commands.spawn(Camera2dBundle::default());

    /* spawn pot to play with */
    spawn_pot(&mut commands, &asset_server);
    // spawn player
    spawn_player(&mut commands, &asset_server, &mut texture_atlases);
}


pub fn spawn_pot(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>
){
    let pot_handle = asset_server.load("pot.png");
    commands.spawn((
        SpriteBundle{
            texture: pot_handle,
            transform: Transform::from_xyz(200.,200.,1.),
            ..default()
        },
        Pot{
            touch: 0
        }
    ));
}