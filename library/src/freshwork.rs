use bevy::prelude::*;
use crate::{cuscuta_resources::*, player::*};

#[derive(Component)]
pub struct Timestamp{
    time: f32
}

#[derive(Component)]
pub struct InputQueue{
    pub q: Vec<(Timestamp, KeyCode),>
}

pub fn update_player(
    mut players: Query<(&Timestamp, &mut Velocity, &mut Transform, &mut InputQueue),With<Player>>,

){
    for (time, mut velocity, mut transform, queue) in players.iter_mut() {
        let curr_time = time.time;
        let mut curr_velo = velocity.into_inner();
        let mut curr_transform = transform.into_inner();
        for(input_time, key) in &queue.q{
            if time.time > input_time.time {
                //queue.q.remove(index)
                //TODO remove
            }
            else {// time <= input_time
                match key{
                    KeyCode::KeyW => (curr_velo, curr_transform) = 
                                        move_north(curr_time,input_time.time
                                                    &curr_velo, &curr_transform),
                    KeyCode::KeyA => (curr_velo, curr_transform) = 
                                        move_west(curr_time, input_time.time), 
                    KeyCode::KeyS => (curr_velo, curr_transform) = 
                                        move_south(curr_time, input_time.time),
                    KeyCode::KeyD => (curr_velo, curr_transform) = 
                                        move_east(curr_time, input_time.time),
                    _ => todo!()
                }
            }
        }
    }
}

fn move_north(
    curr_time:f32,
    input_time:f32,
    velocity:&Velocity,
    transform:&Transform,
    sprinting:bool,
    crouching:bool
) -> (Velocity, Transform) {
    let delta_time: f32 = curr_time - input_time;
    let acceleration = ACCELERATION_RATE * delta_time;
    let max_speed = PLAYER_SPEED;
    if(sprinting){
        acceleration = acceleration * SPRINT_MULTIPLIER;
        max_speed = max_speed * SPRINT_MULTIPLIER;
    }
    if(crouching){
        acceleration = acceleration * CROUCH_MULTIPLIER;
        max_speed = max_speed * CROUCH_MULTIPLIER;
    }

    velocity.velocity = if ()



    return (velocity, transform)
}