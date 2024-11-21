use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::io;
use crate::enemies::{EnemyId, EnemyMovement};
use crate::cuscuta_resources::Health;

#[derive(Component, Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Timestamp{
    pub time: u64
}
impl Timestamp {
    pub fn new(time:u64) -> Self {
        Self{
            time:time
        }
    }

}

/* simple stupid lamport clock. really just need
 * to increment everytime we have a significant 'event'
 * which still needs to be defined (send/recv?? aka fixed tickrate)
 * Lamport works on our implementation being correct. Always
 * assign when we recieve a packet, and then geti (maybe need to switch
 * my ordering of events there, might want the +1 return) ANYWAYS
 * and every event must increment (event for now can be FIXED TICKS).
 * If we assign everytime we recieve, we can ensure ordering of events.
 * When server sends back a 'past state', we should sttach the clock of
 * the tick at which tha client sent state, alongside a typical sequence 
 * with the server send. We can use these sequence numbers as ticks! 
 * In our update player functionality, we can assume that inputs occurred 
 * for the whole tick, and if theres a 2 tick gap (like we have 4, 6 but no 5),
 * treat it as whatever, we don't care. We will have empty inputs
 * if we truly didn't touch a key, this gap i was speaking of
 * probably occurs from clock updates from server or even other client
 * working it's way over (all come from server lol) */
#[derive(Resource)]
pub struct Sequence{
    num: u64
}

impl Sequence{
    /* everytime we use a sequence # we should increment 
     * GAHHHH HMAYBE WE SHOULD SWITCH ORDERING REALLY JUST
     * DEPENDS ON HOW WE USE IT BUT IDK WHATS EASIER */ 
    pub fn geti(&mut self) -> u64{
        self.num += 1;
        self.num - 1
    }

    /* simple get */
    pub fn get(&mut self) -> u64{
        self.num
    }
    
    /* takes the greater value and uses it */
    pub fn assign(&mut self, val:u64){
        if val > self.num{
            self.num = val;
        }
        //else nothing, keep old. We call geti after teebs
    }
    
    pub fn new() -> Self{
        Self{
            num: 0
        }
    }
}


#[derive(Resource, Component)]
pub struct UDP {
    pub socket: UdpSocket,
}

pub struct UserInputAddr { 
    pub user_string: String,
}

#[derive(Resource)]
pub struct BufSerializer {
    pub serializer: FlexbufferSerializer,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerC2S{
    pub head: Header,
    pub key: Vec<KeyCode>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerS2C{
    pub head: Header,
    pub transform: Transform,
    pub velocity: Vec2,
    pub health: Health,
    pub crouch: bool,
    pub attack: bool,
    pub roll: bool,
    pub sprint: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MapS2C{
    pub head: Header,
    pub matrix: Vec<Vec<u8>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EnemyS2C{
    pub head: Header,
    pub enemytype: EnemyId,
    pub movement: EnemyMovement,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub head: Header
}
#[derive(Component, Serialize,Deserialize, PartialEq, Debug, Clone)]
pub struct Header{
    pub network_id: u8,
    pub sequence_num: u64,
    pub timestamp: u64
}
impl Header{
    pub fn new(id: u8, seq: u64, time: u64)-> Self{
        Self{
            network_id: id,
            sequence_num: seq,
            timestamp: time,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientPacket{
    PlayerPacket(PlayerC2S),
    IdPacket(IdPacket),
}

#[derive(Serialize, Deserialize)]
pub enum ServerPacket{
    PlayerPacket(PlayerS2C),
    MapPacket(MapS2C),
    IdPacket(IdPacket),
    EnemyPacket(EnemyS2C),
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] { // will slice anything into u8 array 
    
    ::core::slice::from_raw_parts((p as *const T) as *const u8,
     ::core::mem::size_of::<T>())
}

pub unsafe fn u8_to_f32(input_arr: &[u8]) -> (&[u8], &[f32], &[u8]) {
    // prefix, actual stuff, suffix
    input_arr.align_to::<f32>()
}


pub fn get_ip_addr() -> String {
    print!("Enter the IP Address  + enter then port number + enter you would like your socket to bind to: \n");
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer); // read from stdin

    buffer // return buffer

}
