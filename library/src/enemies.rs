use bevy::prelude::*;
use rand::Rng;


use crate::{cuscuta_resources::*, player::*, collision::*};

/* struct to query for */
#[derive(Component)]
pub struct Enemy {
    pub direction: Vec2,
} 

/* Should soon be deprecated. Need to base
 * this off of server information...*/
pub fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..NUMBER_OF_ENEMIES {
        let random_x: f32 = rng.gen_range(-MAX_X..MAX_X);
        let random_y: f32 = rng.gen_range(-MAX_Y..MAX_Y);

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(random_x, random_y, 900.),
                texture: asset_server.load("enemies/skelly.png"),
                ..default()
            },
            Enemy {
                direction: Vec2::new(rng.gen::<f32>(), rng.gen::<f32>()).normalize(),
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
            direction: Vec2::new(rng.gen::<f32>(), rng.gen::<f32>()).normalize()
        },
    ));
}

pub fn enemy_movement(
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
    mut player_query: Query<(&mut Transform, &Player, &mut Health), (With<Player>, Without<Enemy>)>,
    time: Res<Time>
) {
    //let mut desire: [Vec3; NUMBER_OF_ENEMIES as usize] = Default::default();
    //let mut index = 0;
    for (mut transform, _enemy) in enemy_query.iter_mut() {
        
        // checking which player each enemy should follow (if any are in range)
        let mut player_transform: Transform = Transform::from_xyz(0., 0., 0.); //to appease the all-knowing compiler
        //let playerto: Player;
        let mut longest: f32 = 0.0;
        for (mut pt, p, mut ph) in player_query.iter_mut(){
            let hel: Mut<'_, Health> = ph;
            let xdis = (pt.translation.x - transform.translation.x).abs() * (pt.translation.x - transform.translation.x).abs();
            let ydis = (pt.translation.x - transform.translation.x).abs() * (pt.translation.x - transform.translation.x).abs();
            if ydis + xdis < ENEMY_SPOT_DISTANCE * ENEMY_SPOT_DISTANCE {
                if ydis + xdis > longest {
                longest = ydis + xdis;
                player_transform = *pt;
                //playerto = *p;
            }}

            // handling if enemy has hit player
            let enemy_aabb = Aabb::new(transform.translation, Vec2::splat(TILE_SIZE as f32));
            let player_aabb = Aabb::new(pt.translation, Vec2::splat(TILE_SIZE as f32));
            if enemy_aabb.intersects(&player_aabb){
                //ph.current = ph.current - 25.;

                let direction_to_player = player_transform.translation - transform.translation;
                let normalized_direction = direction_to_player.normalize();
                //let opp_direction = Vec3::new(normalized_direction.x * -1., normalized_direction.y * -1., normalized_direction.z);
                pt.translation += normalized_direction * 64.;
                player_transform.translation = pt.translation;
            }

        }

        // if none in range, check for next enemy
        if longest == 0.0{
        //    desire[index] = Vec3::new(0.,0.,0.);
        //    index = index + 1;
            continue;
        }
        
        // finding direction to move
        let direction_to_player = player_transform.translation - transform.translation;
        let normalized_direction = direction_to_player.normalize();

        //desire[index] = normalized_direction;
        //index = index + 1;


    // making sure enemies do not collide with one another
    /*for (mut transform, _enemy) in enemy_query.iter_mut() {
        if othert.translation.x != transform.translation.x && othert.translation.y != transform.translation.y{
            let enemy_aabb = Aabb::new(transform.translation + normalized_direction, Vec2::splat(TILE_SIZE as f32));
            let other_aabb = Aabb::new(othert.translation, Vec2::splat(TILE_SIZE as f32));
            if enemy_aabb.intersects(&other_aabb){
                continue;
            }
        }  **/  

        transform.translation += normalized_direction * ENEMY_SPEED * time.delta_seconds();

    }
}
