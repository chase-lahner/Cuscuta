use bevy::prelude::*;

use crate::{carnage::CarnageBar, collision::{self, *}, cuscuta_resources::*, enemies::Enemy, network, room_gen::*};

#[derive(Component)]
pub struct Player;// wow! it is he!

#[derive(Component)]
pub struct NetworkId {
    pub id: u8, // we will have at most 2 players so no more than a u8 is needed
}

#[derive(Component)]
pub struct Crouch{
    pub crouching: bool
}

#[derive(Component)]
pub struct Sprint{
    pub sprinting:bool
}
/* global boolean to not re-attack */
#[derive(Component)]
pub struct Attack{
    pub attacking: bool
}


#[derive(Bundle)]
pub struct ClientPlayerBundle{
    sprite: SpriteBundle,
    atlas: TextureAtlas,
    animation_timer: AnimationTimer,
    animation_frames: AnimationFrameCount,
    velo: Velocity,
    id: NetworkId,
    player: Player,
    health: Health,
    crouching: Crouch,
    sprinting: Sprint,
    attacking: Attack,
}

#[derive(Bundle)]
pub struct ServerPlayerBundle{
    velo: Velocity,
    id: NetworkId,
    player: Player,
    health: Health,
    crouching: Crouch,
    sprinting: Sprint,
    attacking: Attack
}

pub fn player_attack(
    time: Res<Time>,
    input: Res<ButtonInput<MouseButton>>,
    mut player: Query<
        (
            &Velocity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &AnimationFrameCount,
            &mut Attack
        ),
        With<Player>,
    >,
    mut carnage_q: Query<&mut CarnageBar, With<CarnageBar>>
) {
    /* In texture atlas for ratatta:
     * 0 - 3 = up
     * 4 - 7 = down
     * 8 - 11 = right
     * 12 - 15 = left
     * ratlas. heh. get it.*/
     let (v, mut ratlas, mut timer, _frame_count, mut attack) = player.single_mut();
     let mut carnage = carnage_q.single_mut();
     let abx = v.velocity.x.abs();
     let aby = v.velocity.y.abs();

     if input.just_pressed(MouseButton::Left)
     {
        attack.attacking = true; //set attacking to true to override movement animations
        
        // deciding initial frame for swing (so not partial animation)
        if abx > aby {
            if v.velocity.x >= 0.{ratlas.index = 8;}
            else if v.velocity.x < 0. {ratlas.index = 12;}
        }
        else {
            if v.velocity.y >= 0.{ratlas.index = 0;}
            else if v.velocity.y < 0. {ratlas.index = 4;}
        }
        /* increment carnage. stupid fer now */
        if carnage.carnage < 50.{
            carnage.carnage += 1.;
        }
        timer.reset();
     }
    if attack.attacking == true
    {
        timer.tick(time.delta());

        if abx > aby {
            if v.velocity.x >= 0.{
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 8;}
                if ratlas.index == 11{attack.attacking = false; ratlas.index = 24} //allow for movement anims after last swing frame
            }
            else if v.velocity.x < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 12;}
                if ratlas.index == 15{attack.attacking = false; ratlas.index = 28} //allow for movement anims after last swing frame
            }
        }
        else {
            if v.velocity.y >= 0.{
                if timer.finished(){ratlas.index = (ratlas.index + 1) % 4;}
                if ratlas.index == 3{attack.attacking = false; ratlas.index = 16} //allow for movement anims after last swing frame
            }
            else if v.velocity.y < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 4;}
                if ratlas.index == 7{attack.attacking = false; ratlas.index = 20} //allow for movement anims after last swing frame
            }
        }
    }
}

/* Spawns in player, uses PlayerBundle for some consistency*/
pub fn client_spawn_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let player_sheet_handle = asset_server.load("player/4x8_player.png");
    let player_layout = TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE), 4, 8, None, None);
    let player_layout_len = player_layout.textures.len();
    let player_layout_handle = texture_atlases.add(player_layout);

    // spawn player at origin
    commands.spawn(ClientPlayerBundle{
        sprite: SpriteBundle {
            texture: player_sheet_handle,
            transform: Transform::from_xyz(0., 0., 900.),
            ..default()
        },
        atlas: TextureAtlas {
            layout: player_layout_handle,
            index: 0,
        },
        animation_timer: AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
        animation_frames: AnimationFrameCount(player_layout_len),
        velo: Velocity::new(),
        id:NetworkId {
        id: 0
        },
        player: Player,
        health: Health{
            max: 100.,
            current: 100.
        },
        crouching: Crouch{crouching:false},
        sprinting: Sprint{sprinting:false},
        attacking: Attack{attacking:false}
});
}

/* spawns in a player entity for server. No Gui */
pub fn server_spawn_player(
    commands: &mut Commands,
    id: u8
){
    commands.spawn((
        Velocity::new(),
        NetworkId{
            id: id
        },
        Player,   
    ));
}

/* Checks for player interacting with game world.
 * E for interact? Assumed menu etc. could also
 * fit in here.. I also currently have pot as
 * it's own resource, maybe make an 'interactable'
 * trait for query? - rorto */
pub fn player_interact(
    mut player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
    input: Res<ButtonInput<KeyCode>>,
    mut pot_q: Query<&mut Pot>,
    mut pot_transform_q: Query<&mut Transform, (With<Pot>, Without<Player>)>
){
    let mut pot = pot_q.single_mut();
    let pot_transform = pot_transform_q.single_mut();
    let (player_transform, mut _player_velocity) = player.single_mut();
    /* Has nothing to do with particles */
    let pot_particle_collider = Aabb::new(
        pot_transform.translation, Vec2::splat(TILE_SIZE as f32));
    let player_particle_collider = collision::Aabb::new(
        player_transform.translation, Vec2::splat(TILE_SIZE as f32));

    /* touch is how many frames since pressed
     * We only want to increment if not pressed
     * recently */
    if input.just_pressed(KeyCode::KeyE)
        && pot_particle_collider.intersects(&player_particle_collider)
        && pot.touch == 0
    {
        pot.touch += 1;

    }


}

pub fn player_input(
    mut commands: Commands,
    mut player: Query<( &mut Velocity, &mut Crouch, &mut Sprint), (With<Player>, Without<Background>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,

) {
    let (mut player_velocity, mut crouch_query, mut sprint_query) = player.single_mut();
    /* should be copy of player for us to apply input to */
    let mut deltav = player_velocity.velocity;


    /* first check if we sprint or crouch for gievn frame */
    let sprint = sprint_query.as_mut();
    let sprint_multiplier = if input.pressed(KeyCode::ShiftLeft) {
        sprint.sprinting = true;
        SPRINT_MULTIPLIER
    } else {
        sprint.sprinting = false;
        1.0
    };

    /* check if crouched */
    let crouch = crouch_query.as_mut();
    let crouch_multiplier = if input.pressed(KeyCode::KeyC){
        crouch.crouching = true;
        CROUCH_MULTIPLIER
    } else {
        crouch.crouching = false;
        1.0
    };

    /* We have a fixed acceleration rate per time t, this
     * lets us know how long it has been sine we updated,
     * allowing us for smooth movement even when frames
     * fluctuate */
    let deltat = time.delta_seconds();
    /* base acceleration * time gives standard movement. 
     * crouch and sprint either halv, double, or cancel each other out*/
    let acceleration = ACCELERATION_RATE * deltat * crouch_multiplier * sprint_multiplier;
    let current_max = PLAYER_SPEED * crouch_multiplier * sprint_multiplier;



    /* Take in keypresses and translate to velocity change
     * We have a max speeed of max_speed based off of crouch/sprint, 
     * and each frame are going to accelerate towards that, via acceleration */

    /* God. im about to make it all 8 cardinals so we dont speed
     * up on the diagonals TODODODODODODODODO */
    if input.pressed(KeyCode::KeyA){
        deltav.x -= acceleration;
    }
    if input.pressed(KeyCode::KeyD) {
        deltav.x += acceleration;
    }
    if input.pressed(KeyCode::KeyW) {
        deltav.y += acceleration;
    }
    if input.pressed(KeyCode::KeyS) {
        deltav.y -= acceleration;
    }

    /* We now must update the player using the information we just got */

   /* now we chek if we are going to fast. This doessss include
    * a sqrt... if someone could figure without would be loverly */
    let mut adjusted_speed = deltav.length();
    
    if adjusted_speed > current_max{
        /* here we are. moving too fast. Honestly, I don't
         * think that we should clamp, we may have just crouched.
         * We should decelerate by a given rate, our acceleration rate! s
         * not using the adjusted, dont want if crouch slow slowdown yk */
        adjusted_speed -= ACCELERATION_RATE;
        deltav.clamp_length_max(adjusted_speed);
    }

    /* final set */
    player_velocity.velocity = deltav;
    
}

/* Old setup had too much in one function, should collision check be
 * done in here??? */
pub fn update_player_position(
    time: Res<Time>,
    mut players: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
){
    /* We use delta time to determine ur velocity earlier, so we really want to use it again here?
     * It gives second since update, not since we got input... */
    for( mut transform, mut velocity) in players.iter_mut(){
        transform.translation.x += velocity.velocity.x * time.delta_seconds();
        transform.translation.y += velocity.velocity.y * time.delta_seconds();

        let mut hit_door = false;
        // take care of horizontal and vertical movement + enemy collision check
        // TODODODODODODODODODODODO

        // if we hit a door
        // if hit_door {
        //     transition_map(&mut _commands, &_asset_server, &mut room_manager, room_query, &mut pt); // Pass room_query as argument
        // }
    }
}



/* hopefully deprecated soon ^^ new one ^^
 * lil too much going on here, must be broke down */
pub fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
    mut room: Query<&mut Transform, (Without<Player>, Without<Enemy>)>,
    mut room_manager: ResMut<RoomManager>,
    mut _commands: Commands, 
    _asset_server: Res<AssetServer>, 
    room_query: Query<Entity, With<Room>>, 
) {

    let (mut pt, mut pv) = player.single_mut();
    let mut deltav = Vec2::splat(0.);

    // INPUT HANDLING
    if input.pressed(KeyCode::KeyA) {
        deltav.x -= 1.;
    }
    if input.pressed(KeyCode::KeyD) {
        deltav.x += 1.;
    }
    if input.pressed(KeyCode::KeyW) {
        deltav.y += 1.;
    }
    if input.pressed(KeyCode::KeyS) {
        deltav.y -= 1.;
    }

    let deltat = time.delta_seconds();
    let acc = ACCELERATION_RATE * deltat;

    // sprint - check if shift is pressed
    let speed_multiplier = if input.pressed(KeyCode::ShiftLeft) {
        SPRINT_MULTIPLIER
    } else {
        1.0
    };

    // set new max speed
    let max_speed = PLAYER_SPEED * speed_multiplier;


    pv.velocity = if deltav.length() > 0. {
        (pv.velocity + (deltav.normalize_or_zero() * acc)).clamp_length_max(max_speed)
    } else if pv.velocity.length() > acc {
        pv.velocity + (pv.velocity.normalize_or_zero() * -acc)
    } else {
        Vec2::splat(0.)
    };


    let change = pv.velocity * deltat;

    let mut hit_door: bool = false;

    let (room_width, room_height) = room_manager.current_room_size();

    // Calculate new player position and clamp within room boundaries
    let new_pos_x = (pt.translation.x + change.x)
        .clamp(-room_width / 2.0 + TILE_SIZE as f32 + TILE_SIZE as f32 / 2.0,
         room_width / 2.0 - TILE_SIZE as f32 - TILE_SIZE as f32 / 2.0);
    let new_pos_y = (pt.translation.y + change.y)
        .clamp(-room_height / 2.0 + TILE_SIZE as f32 + TILE_SIZE as f32 / 2.0,
             room_height / 2.0 - TILE_SIZE as f32 - (TILE_SIZE / 2) as f32 / 2.0);

    pt.translation.x = new_pos_x;
    pt.translation.y = new_pos_y;


    // take care of horizontal and vertical movement + enemy collision check
    handle_movement_and_enemy_collisions(
        &mut pt, 
        change, 
        &mut hit_door, 
        &mut enemies,
        &mut room_manager, 
    );


    // if we hit a door
    if hit_door {
        transition_map(&mut _commands, &_asset_server, &mut room_manager, room_query, &mut pt); // Pass room_query as argument
    }
}


pub fn handle_movement_and_enemy_collisions(
    pt: &mut Transform,
    change: Vec2,
    hit_door: &mut bool,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
    room_manager: &mut RoomManager,
) {
    // Calculate new player position
    let new_pos = pt.translation + Vec3::new(change.x, change.y, 0.0);
    let player_aabb = collision::Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    // Translate player position to grid indices
    let (_topleft, _topright, _bottomleft, _bottomright) = translate_coords_to_grid(&player_aabb, room_manager);

     // Translate player position to grid indices
     let grid_x = (new_pos.x / TILE_SIZE as f32).floor();
     let grid_y = (new_pos.y / TILE_SIZE as f32).floor();
     //println!("Player grid position: x = {}, y = {}", grid_x, grid_y);

    // Handle collisions and movement within the grid
    handle_movement(pt, Vec3::new(change.x, 0., 0.), room_manager, hit_door, enemies);
    handle_movement(pt, Vec3::new(0., change.y, 0.), room_manager, hit_door, enemies);
}


pub fn handle_movement(
    pt: &mut Transform,
    change: Vec3,
    room_manager: &mut RoomManager,
    hit_door: &mut bool,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    let new_pos = pt.translation + change;
    let player_aabb = collision::Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    // Get the current room's grid size (room width and height)
    let current_grid = room_manager.current_grid();
    let room_width = current_grid.len() as f32 * TILE_SIZE as f32;
    let room_height = current_grid[0].len() as f32 * TILE_SIZE as f32;

    let (topleft, topright, bottomleft, bottomright) = translate_coords_to_grid(&player_aabb, room_manager);

    // check for collisions with enemies
    for enemy_transform in enemies.iter() {
        let enemy_aabb = Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if player_aabb.intersects(&enemy_aabb) {
            // handle enemy collision here (if necessary)
            return;
        }
    }

    // movement within bounds and wall/door collision check
    if new_pos.x >= -room_width / 2.0 + TILE_SIZE as f32 / 2. &&
        new_pos.x <= room_width / 2.0 - TILE_SIZE as f32 / 2. &&
        new_pos.y >= -room_height / 2.0 + TILE_SIZE as f32 / 2. &&
        new_pos.y <= room_height / 2.0 - TILE_SIZE as f32 / 2. &&
        topleft != 1 && topright != 1 && bottomleft != 1 && bottomright != 1
    {
        pt.translation = new_pos;
    }

    // check for door transition
    if topleft == 2 || topright == 2 || bottomleft == 2 || bottomright == 2 {
        *hit_door = true;
    }
}

pub fn animate_player(
    time: Res<Time>,
    mut player: Query<
        (
            &Velocity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &AnimationFrameCount,
            &Attack
        ),
        With<Player>,
    >,
) {
    /* In texture atlas for ratatta:
     * 16 - 19 = up
     * 20 - 23 = down
     * 24 - 27 = right
     * 28 - 31 = left
     * ratlas. heh. get it.*/
    let (v, mut ratlas, mut timer, _frame_count, attack) = player.single_mut();
    if attack.attacking == true{return;}//checking if attack animations are running
    //if v.velocity.cmpne(Vec2::ZERO).any() {
        timer.tick(time.delta());

        let abx = v.velocity.x.abs();
        let aby = v.velocity.y.abs();

        if abx > aby {
            if v.velocity.x > 0.{
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 24;}
            }
            else if v.velocity.x < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 28;}
            }
        }
        else {
            if v.velocity.y > 0.{
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 16;}
            }
            else if v.velocity.y < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 20;}
            }
        }
    //}
}