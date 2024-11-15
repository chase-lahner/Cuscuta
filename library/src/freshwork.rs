use std::{ops::Mul, time::Instant};

use bevy::prelude::{KeyCode::*, *};
use serde::{Serialize, Deserialize};
use crate::{ui::CarnageBar, cuscuta_resources::*, player::*};



pub fn update_player(
    mut player_q: Query<(&Timestamp, &mut Velocity, &mut Transform, &mut InputQueue, &Crouch, &Sprint),With<Player>>,
    mut carange_q: Query<&mut CarnageBar>
){
    /* query establihsed, not active state */
    for (time, mut velocity, mut transform, queue, crouch, sprint) in player_q.iter_mut() {
        let mut curr_time: u128 = time.time; // in nanoseconds
        let mut curr_velo = velocity.into_inner();
        let mut curr_transform = transform.into_inner();
        for(input_time, key) in &queue.q{
            if time.time > input_time.time + 1.{// 
                //queue.q.remove(index)
                //TODO remove
            }
            else if time.time <= input_time.time{// time <= input_time
                match key{
                    KeyW | KeyA | KeyS | KeyD 
                    => (*curr_velo, *curr_transform) = 
                        move_over(curr_time,input_time.time,
                        curr_velo, curr_transform,
                        sprint.sprinting, crouch.crouching,
                        *key),
                    CapsLock => crouchy(),
                    ShiftLeft => roll(),
                    KeyQ | KeyE => item_rotate(),// how are we doing items?
                    Space => attack(),
                    _ => todo!()//more keypresses! more actions!
                }
            }
            //curr time is not accurate atm, it uses last commands, not last
            //move etc etc
            curr_time = input_time.time;
        }//end input_queue
        //more clapms? last minute checks on collision?????????
    }
}

/* move player a smidge up, called on keypress "W" */
fn move_over(
    curr_time:u128,
    input_time:u128,
    velocity:&mut Velocity,
    transform:&mut Transform,
    sprinting:bool,
    crouching:bool,
    key:KeyCode
) -> (Velocity, Transform) {


    /* calulate time between last input used */
    let delta_time: u128 = curr_time - input_time;
    /* Use said time to calculate estimated acceleration rate */
    let mut acceleration: <u128 as Mul<u128>>::Output = ACCELERATION_RATE as u128 * delta_time;
    let mut max_speed = PLAYER_SPEED;
    let mut delta_velo = Vec2::splat(0.);

    /* Aply sprint/ crouch */
    if sprinting {
        acceleration = acceleration * SPRINT_MULTIPLIER as u128;
        max_speed = max_speed * SPRINT_MULTIPLIER;
    }
    if crouching {
        acceleration = acceleration * CROUCH_MULTIPLIER as u128;
        max_speed = max_speed * CROUCH_MULTIPLIER;
    }

    /* Apply keypress */
    match key{
        KeyW => delta_velo.y +=1.,
        KeyA => delta_velo.x -=1.,
        KeyS => delta_velo.y -=1.,
        KeyD => delta_velo.x +=1.,
        _ => todo!()
    }
    /* apply acceleration to velocity */
    velocity.velocity = if delta_velo.length() > 0. {
        (velocity.velocity + (delta_velo.normalize_or_zero() 
                * acceleration)).clamp_length_max(max_speed)
    } else if velocity.velocity.length() > acceleration {
        velocity.velocity + (velocity.velocity.normalize_or_zero() * -acceleration)
    } else{
        Vec2::splat(0.)
    };

    /* use velocity to calculate distance travelled */
    let change = velocity.velocity * delta_time;

    /* unclamped at the moment. should do our collision work here before
     * creating position. Last implementation of move clamped to room bound but
     * we should do it based on ACTUAL collision not assumed  */
    let new_pos_x = (transform.translation.x + change.x);//.clamp();
    let new_pos_y = transform.translation.y + change.y;

    /* set em up */
    transform.translation.x = new_pos_x;
    transform.translation.y = new_pos_y;

    return (velocity.clone(), *transform)
}

fn roll(){}

fn crouchy(){}

fn item_rotate(){}

/* Differences betwen client/server?  */
fn attack(){}

