use std::ops::Mul;
use bevy::prelude::{KeyCode::*, *};
use crate::{ui::CarnageBar, cuscuta_resources::*, player::*, network::Timestamp};



pub fn update_player(
    mut player_q: Query<(&Timestamp, &mut Velocity, &mut Transform, &mut InputQueue, &Crouch, &Sprint),With<Player>>,
    mut carange_q: Query<&mut CarnageBar>
){
    /* query establihsed, not active state */
    for (time, mut velocity, mut transform, queue, crouch, sprint) in player_q.iter_mut() {
        let mut curr_time: u64 = time.time; // in nanoseconds
        let mut curr_velo = velocity.into_inner();
        let mut curr_transform = transform.into_inner();
        for(input_time, key) in &queue.q{
            if time.time > input_time.time + 1{// 
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
    curr_time:u64,
    input_time:u64,
    velocity:&mut Velocity,
    transform:&mut Transform,
    sprinting:bool,
    crouching:bool,
    key:KeyCode
) -> (Velocity, Transform) {
    /* TODODODODOD Idk if this logic is very sound...
     * I'm a little worried about how we use time... We definitely
     * need to know how long it's been since the last udpdate (bro we can soooo
     * fix the client to 60fps tbh that might make our lives so much easier) ANYWAYS
     * the way we are using time rn is iffy... GAHHH im overthinking...
     * This whole blurb spawned out of my desire to use move_over() every key when
     * we recieve data from client buuuuuut tbh we should just take it in and
     * then do our standard update_player(), which calls this as needed.. 
     * I shall return here at some point im sure */

    /* calulate time between last input used */
    let delta_time: u64 = curr_time - input_time;
    /* Use said timedelta to calculate estimated acceleration rate */
    let mut acceleration: <f32 as Mul<f32>>::Output = ACCELERATION_RATE * delta_time as f32;
    let mut max_speed = PLAYER_SPEED;
    let mut delta_velo = Vec2::splat(0.);

    /* Aply sprint/ crouch */
    if sprinting {
        acceleration = acceleration * SPRINT_MULTIPLIER as f32;
        max_speed = max_speed * SPRINT_MULTIPLIER;
    }
    if crouching {
        acceleration = acceleration * CROUCH_MULTIPLIER as f32;
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
    let change = velocity.velocity * delta_time as f32;

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

