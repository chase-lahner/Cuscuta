use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::io;
use crate::enemies::{EnemyId, EnemyMovement};
use crate::cuscuta_resources::Health;
use crate::ui::CarnageBar;


/* Packets queues are used to hold packets when creted, before
 * being sent. We will send every packet in the corresponding queue
 * once every fixedupdate (currently 60hz) */
#[derive(Resource, Debug)]
pub struct ServerPacketQueue{
    pub packets: Vec<ServerPacket>
}
impl ServerPacketQueue{
    pub fn new() -> Self{
        Self{
            packets: Vec::new()
        }
    }
}

#[derive(Resource, Debug)]
pub struct ClientPacketQueue{
    pub packets: Vec<ClientPacket>
}
impl ClientPacketQueue{
    pub fn new() -> Self{
        Self{
            packets: Vec::new()
        }
    }
}

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

/* simple stupid vector lamport clock. really just need
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
 * working it's way over (all come from server lol)
 * - roto */


/* another aside on sequences, I think we should increment them AFTER we send.
 * We want to be able to user a sequence value for the entirety of the tick, so...
 * yah. Also think when assigning, we should +1 because event A of sending must
 * happen BEFORE event B of receiving (A -> B) a la lamport  */

/* Shifted towards vector clock, all above should stay the same, we are just
 * now trying to use one clock value per interconnected process, and our index
 * lets us know which value if ours. Client don't really care, but Server might
 * a lil bit */
#[derive(Resource,Serialize,Deserialize, PartialEq, Debug, Clone)]
pub struct Sequence{
    pub nums: Vec<u64>,
    index: usize,
}

impl Sequence{

    /* everytime we use a sequence # we should increment 
     * so this returns num+1 and does that work*/ 
    pub fn geti(&mut self) -> u64{
        self.nums[self.index] += 1;
        self.nums[self.index]
    }

    /* changes index value */
    pub fn new_index(&mut self, index:usize){
        self.index = index;
        while index+1 > self.nums.len(){
            self.nums.push(0);
        }
    }
    /* simple get, our index if where WE are in vec.
     * gets OUR sequence # */
    pub fn get(& self) -> u64{
        self.nums[self.index]
    }
    
    /* assigns any value to it's greater counterpart within
     * the Vec */
    pub fn assign(&mut self, other:&Sequence){
        /* they have more than we do! */
        let other_len: usize = other.nums.len();
        let mut my_len: usize = self.nums.len();
        /* iterate over, making sure we have '0' spaces for new clock values */
        if other_len > my_len{
            while my_len < other_len+1{
                self.nums.push(0);
                my_len+=1;
            }
        }
        /* for all elements in self */
        for i in 0..other.nums.len(){
            /* if we are less, increment */
            if self.nums[i] < other.nums[i]{
                self.nums[i] = other.nums[i];
            }
        }
        /* we may want to increment self.nums[self.index] here,
         * as this is called to confirm a recv, buttttt maybe not? */
    }
    
    pub fn new(index:usize) -> Self{
        let mut vec = Vec::new();
        while vec.len() < index+1{vec.push(0)}
        Self{
            nums: vec,
            index: index 
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
pub struct PlayerSendable{
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
    pub size: (f32, f32),
    pub max: (f32, f32),
    pub z: f32,
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EnemyS2C{
    pub transform: Transform,
    pub head: Header,
    pub enemytype: EnemyId,
    pub movement: EnemyMovement,
    pub health: Health,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub head: Header
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct KillEnemyPacket{
    pub enemy_id: EnemyId,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]

pub struct MonkeyPacket{
    pub head: Header,
    pub transform: Transform,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DespawnAllPacket {
    pub kill: bool
}

impl DespawnAllPacket{
    pub fn new() -> Self{
        Self{
            kill: true,
        }
    }
}

#[derive(Component, Serialize,Deserialize, PartialEq, Debug, Clone)]
pub struct Header{
    pub network_id: u8,
    pub sequence: Sequence,
}
impl Header{
    pub fn new(id: u8, seq: Sequence)-> Self{
        Self{
            network_id: id,
            sequence: seq,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecreaseEnemyHealthPacket{
    pub enemy_id: EnemyId,
    pub decrease_by: f32,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct CarnagePacket{
    pub carnage: CarnageBar,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientPacket{
    PlayerPacket(PlayerSendable),
    IdPacket(IdPacket),
    KillEnemyPacket(KillEnemyPacket),
    DecreaseEnemyHealthPacket(DecreaseEnemyHealthPacket),
    MonkeyPacket(MonkeyPacket),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket{
    PlayerPacket(PlayerSendable),
    MapPacket(MapS2C),
    IdPacket(IdPacket),
    EnemyPacket(EnemyS2C),
    DespawnPacket(KillEnemyPacket),
    MonkeyPacket(MonkeyPacket),
    DespawnAllPacket(DespawnAllPacket),
    CarnagePacket(CarnagePacket),
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
