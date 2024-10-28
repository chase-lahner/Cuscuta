use bevy::prelude::*;
use rand::Rng;


use crate::{cuscuta_resources::*, player::*, collision::*};

/* struct to query for */
#[derive(Component)]
pub struct Enemy {
    pub direction: Vec2,
    pub timer: Timer,
    pub axis: i32,
    pub lastseen: Vec3,
} 

/* Should soon be deprecated. Need to base
 * this off of server information...*/
pub fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..NUMBER_OF_ENEMIES {
        let random_x: f32 = rng.gen_range((-MAX_X + 64.)..(MAX_X - 64.));
        let random_y: f32 = rng.gen_range((-MAX_Y + 64.)..(MAX_Y - 64.));

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(random_x, random_y, 900.),
                texture: asset_server.load("enemies/skelly.png"),
                ..default()
            },
            Enemy {
                direction: Vec2::new(rng.gen::<f32>(), rng.gen::<f32>()).normalize(),
                timer: Timer::from_seconds(3.0, TimerMode::Repeating),
                axis: 1,
                lastseen: Vec3::new(99999.,0.,0.)
            },
        ));
    }

}
pub fn server_spawn_enemy(
    mut commands: Commands
){
    let mut rng = rand::thread_rng();
    commands.spawn((
        Enemy{
            direction: Vec2::new(rng.gen::<f32>(), rng.gen::<f32>()).normalize(),
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            axis: 1,
            lastseen: Vec3::new(99999.,0.,0.)
        },
    ));
}

pub fn enemy_movement(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    mut player_query: Query<(&mut Transform, &Player, &mut Health), (With<Player>, Without<Enemy>)>,
    wall_query: Query<(&Transform, &Wall), (Without<Player>, Without<Enemy>)>, 
    time: Res<Time>
) {
    // for every enemy
    for (mut transform, mut _enemy) in enemy_query.iter_mut() {
        
        // checking which player each enemy should follow (if any are in range)
        let mut player_transform: Transform = Transform::from_xyz(0., 0., 0.); //to appease the all-knowing compiler
        // checking which player is closest
        let mut longest: f32 = 99999999999.0;
        // for every player
        for (mut pt, p, mut ph) in player_query.iter_mut(){
            let xdis = (pt.translation.x - transform.translation.x).abs() * (pt.translation.x - transform.translation.x).abs();
            let ydis = (pt.translation.y - transform.translation.y).abs() * (pt.translation.y - transform.translation.y).abs();
            if ydis + xdis < ENEMY_SPOT_DISTANCE * ENEMY_SPOT_DISTANCE {
                
                let mut blocked = false;
                //line of sight
                for a in 0..20 {
                    let dec = (a as f32)/20.;
                    let xnew = transform.translation.x + dec * (pt.translation.x - transform.translation.x);
                    let ynew = transform.translation.y + dec * (pt.translation.y - transform.translation.y);
                    let pointaabb = Aabb::new(Vec3::new(xnew, ynew, 0.), Vec2::splat(1.));
                    for (wt, w) in wall_query.iter() {
                        //if wt.translation.z == pt.translation.z || wt.translation.z == pt.translation.z - 0.1 {
                            let wallaabb = Aabb::new(wt.translation, Vec2::splat(TILE_SIZE as f32));
                            if pointaabb.losintersect(&wallaabb){
                                blocked = true;
                            }
                        //}
                    }
                }     
                if blocked == true{continue;}

                if ydis + xdis < longest {
                longest = ydis + xdis;
                player_transform = *pt;
                }
            }

            // handling if enemy has hit player
            let enemy_aabb = Aabb::new(transform.translation, Vec2::splat(TILE_SIZE as f32));
            let player_aabb = Aabb::new(pt.translation, Vec2::splat(TILE_SIZE as f32));
            if enemy_aabb.intersects(&player_aabb){
                ph.current -= 5.;

                let direction_to_player = player_transform.translation - transform.translation;
                let normalized_direction = direction_to_player.normalize();
                //let opp_direction = Vec3::new(normalized_direction.x * -1., normalized_direction.y * -1., normalized_direction.z);
                pt.translation.x += normalized_direction.x * 64.;
                pt.translation.y += normalized_direction.y * 64.;
                player_transform.translation = pt.translation;
            }

        }
        _enemy.timer.tick(time.delta());
        // if none in range, patrol and move to next enemy
        if longest == 99999999999.0{       
            
            if _enemy.timer.finished(){
                _enemy.axis = _enemy.axis * -1;
            }
            
            let normalized_direction: Vec3;
            //before patrol, try to go to last seen if have one
            if _enemy.lastseen.x != 99999. {
                let direction_to_player = _enemy.lastseen - transform.translation;
                normalized_direction = direction_to_player.normalize();
                if (_enemy.lastseen.x - transform.translation.x).abs() < 20. || (_enemy.lastseen.y - transform.translation.y).abs() < 20.{
                    _enemy.lastseen.x = 99999.
                }
            }
            else{
            normalized_direction = Vec3::new(1. * _enemy.axis as f32, 0. * _enemy.axis as f32, 0.);}

            //collision detection
            //let mut collide = false;
            let xtemp = transform.translation.x + normalized_direction.x * ENEMY_SPEED/2. * time.delta_seconds();
            let ytemp = transform.translation.y + normalized_direction.y * ENEMY_SPEED/2. * time.delta_seconds();
            let mut xmul: f32 = 1.;
            let mut ymul: f32 = 1.;
            let tempaabb = Aabb::new(Vec3::new(xtemp, ytemp, 0.), Vec2::splat(TILE_SIZE as f32));
            for (wt, w) in wall_query.iter() {
                //if wt.translation.z == player_transform.translation.z || wt.translation.z == player_transform.translation.z - 0.1 {
                    let wallaabb = Aabb::new(wt.translation, Vec2::splat(TILE_SIZE as f32));
                    if tempaabb.intersects(&wallaabb){
                        //collide = true;
                        let tempxaabb = Aabb::new(Vec3::new(xtemp+16., ytemp, 0.), Vec2::splat(1.));
                        let tempx2aabb = Aabb::new(Vec3::new(xtemp-16., ytemp, 0.), Vec2::splat(1.));
                        if tempxaabb.losintersect(&wallaabb) || tempx2aabb.losintersect(&wallaabb){
                            xmul = 0.;
                        }
                        let tempyaabb = Aabb::new(Vec3::new(xtemp, ytemp+16., 0.), Vec2::splat(1.));
                        let tempy2aabb = Aabb::new(Vec3::new(xtemp, ytemp-16., 0.), Vec2::splat(1.));
                        if tempyaabb.losintersect(&wallaabb) || tempy2aabb.losintersect(&wallaabb){
                            ymul = 0.;
                        }
                    }
                //}
            }
            //if collide == true{continue;}

            transform.translation.x += normalized_direction.x * ENEMY_SPEED/2. * time.delta_seconds() * xmul;
            transform.translation.y += normalized_direction.y * ENEMY_SPEED/2. * time.delta_seconds() * ymul;
            continue;
        }
        
        // finding direction to move
        let direction_to_player = player_transform.translation - transform.translation;
        let normalized_direction = direction_to_player.normalize();

        // saving last seen position
        _enemy.lastseen = player_transform.translation;


    // making sure enemies do not collide with one another
    /*for (mut transform, _enemy) in enemy_query.iter_mut() {
        if othert.translation.x != transform.translation.x && othert.translation.y != transform.translation.y{
            let enemy_aabb = Aabb::new(transform.translation + normalized_direction, Vec2::splat(TILE_SIZE as f32));
            let other_aabb = Aabb::new(othert.translation, Vec2::splat(TILE_SIZE as f32));
            if enemy_aabb.intersects(&other_aabb){
                continue;
            }
        }  **/  

        //wall collision detection
        //let mut collide = false;
        let xtemp = transform.translation.x + normalized_direction.x * ENEMY_SPEED * time.delta_seconds();
        let ytemp = transform.translation.y + normalized_direction.y * ENEMY_SPEED * time.delta_seconds();
        let mut xmul: f32 = 1.;
        let mut ymul: f32 = 1.;
        let tempaabb = Aabb::new(Vec3::new(xtemp, ytemp, 0.), Vec2::splat(TILE_SIZE as f32));
        for (wt, w) in wall_query.iter() {
            //if wt.translation.z == player_transform.translation.z || wt.translation.z == player_transform.translation.z - 0.1 {
                let wallaabb = Aabb::new(wt.translation, Vec2::splat(TILE_SIZE as f32));
                if tempaabb.intersects(&wallaabb){
                    //collide = true;
                    let tempxaabb = Aabb::new(Vec3::new(xtemp+16., ytemp, 0.), Vec2::splat(1.));
                    let tempx2aabb = Aabb::new(Vec3::new(xtemp-16., ytemp, 0.), Vec2::splat(1.));
                    if tempxaabb.losintersect(&wallaabb) || tempx2aabb.losintersect(&wallaabb){
                        xmul = 0.;
                    }
                    let tempyaabb = Aabb::new(Vec3::new(xtemp, ytemp+16., 0.), Vec2::splat(1.));
                    let tempy2aabb = Aabb::new(Vec3::new(xtemp, ytemp-16., 0.), Vec2::splat(1.));
                    if tempyaabb.losintersect(&wallaabb) || tempy2aabb.losintersect(&wallaabb){
                        ymul = 0.;
                    }
                }
            //}
        }
        //if collide == true{continue;}


        //transform.translation += normalized_direction * ENEMY_SPEED * time.delta_seconds();
        transform.translation.x += normalized_direction.x * ENEMY_SPEED * time.delta_seconds() * xmul;
        transform.translation.y += normalized_direction.y * ENEMY_SPEED * time.delta_seconds() * ymul;

    }
}