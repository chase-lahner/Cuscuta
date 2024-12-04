use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::collections::VecDeque;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::network::Sequence;
use crate::{
    collision::{self, *},
    cuscuta_resources::*,
    enemies::Enemy,
    network::PlayerSendable,
    room_gen::*,
    ui::CarnageBar,
};

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Player; // wow! it is he!

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct NetworkId {
    pub id: u8, // we will have at most 2 players so no more than a u8 is needed
    pub addr: SocketAddr,
}
impl NetworkId {
    pub fn new(id: u8) -> Self {
        Self {
            id: id,
            /* stupid fake NULL(not really) ass address. dont use this. is set when connection established */
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        }
    }
    /* one good thing about java is reusing fn name*/
    pub fn new_s(id: u8, sock: SocketAddr) -> Self {
        Self { id: id, addr: sock }
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Crouch {
    pub crouching: bool,
}
impl Crouch {
    pub fn new() -> Self {
        Self { crouching: false }
    }
    pub fn new_set(b:bool) ->Self{
        Self{
            crouching:b
        }
    }
    pub fn set(&mut self, crouch: bool){
        self.crouching = crouch;
    }
}

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Roll {
    pub rolling: bool,
}
impl Roll {
    pub fn new() -> Self {
        Self { rolling: false }
    }
    pub fn new_set(b:bool) -> Self{
        Self{
            rolling: b
        }
    }
    pub fn set(&mut self, roll: bool){
        self.rolling = roll;
    }
 }

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Sprint {
    pub sprinting: bool,
}
impl Sprint {
    pub fn new() -> Self {
        Self { sprinting: false }
    }
    pub fn new_set(b: bool) -> Self{
        Self{
            sprinting:b
        }
    }
    pub fn set(&mut self, sprint: bool){
        self.sprinting = sprint;
    }
}
/* global boolean to not re-attack */
#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Attack {
    pub attacking: bool,
}
impl Attack {
    pub fn new() -> Self {
        Self { attacking: false }
    }
    pub fn new_set(b:bool) -> Self{
        Self{
            attacking:b
        }
    }
    pub fn set(&mut self, att: bool){
        self.attacking = att;
    }
}

#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct InputQueue {
    pub q: Vec<(u64, Vec<KeyCode>)>,
}

impl InputQueue {
    pub fn new() -> Self {
        Self { q: Vec::new() }
    }
}

/* pub */
#[derive(Bundle)]
pub struct ClientPlayerBundle {
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub animation_timer: AnimationTimer,
    pub animation_frames: AnimationFrameCount,
    pub velo: Velocity,
    pub id: NetworkId,
    pub player: Player,
    pub health: Health,
    pub crouching: Crouch,
    pub rolling: Roll,
    pub sprinting: Sprint,
    pub attacking: Attack,
    pub inputs: InputQueue,
    pub states: PastStateQueue
}

#[derive(Bundle, Serialize, Deserialize)]
pub struct ServerPlayerBundle {
    pub id: NetworkId,
    pub velo: Velocity,
    pub transform: Transform,
    pub health: Health,
    pub crouching: Crouch,
    pub rolling: Roll,
    pub sprinting: Sprint,
    pub attacking: Attack,
    pub player: Player
    //pub inputs: InputQueue,
    //pub time: Timestamp,
}

#[derive(Component)]
pub struct EnemyPastStateQueue{
    pub q: VecDeque<EnemyPastState>

}

impl EnemyPastStateQueue{
    pub fn new() -> Self{
        Self{
            q: VecDeque::with_capacity(2)
        }
    }
}


pub struct EnemyPastState{
    pub transform: Transform,
}


#[derive(Component, Serialize, Deserialize)]
pub struct PastStateQueue{
    pub q: VecDeque<PastState> // double ended queue, will wrap around when full
}

impl PastStateQueue{
    pub fn new() -> Self{
        Self{
            q: VecDeque::with_capacity(2) // store current and previous states
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct PastState{
    pub velo: Velocity,
    pub transform: Transform,
    pub crouch: Crouch,
    pub roll: Roll,
    pub attack: Attack,
    pub seq: Sequence,
}

impl PastState{
    pub fn new() -> Self{
        Self{
            velo: Velocity::new(),
            transform: Transform::from_xyz(0.,0.,0.),
            crouch: Crouch::new(),
            roll: Roll::new(),
            attack: Attack::new(),
            seq: Sequence::new(0)
        }
    }
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
            &mut Attack,
            &NetworkId,
        ),
        With<Player>,
    >,
    mut carnage_q: Query<&mut CarnageBar, With<CarnageBar>>,
    client_id: Res<ClientId>,
) {
    /* In texture atlas for ratatta:
     * 0 - 3 = up
     * 4 - 7 = down
     * 8 - 11 = right
     * 12 - 15 = left
     * ratlas. heh. get it.*/
    for (v, mut ratlas, mut timer, _frame_count, mut attack, id) in player.iter_mut() {
        if id.id == client_id.id {
            let mut carnage = carnage_q.single_mut();
            let abx = v.velocity.x.abs();
            let aby = v.velocity.y.abs();

            if input.just_pressed(MouseButton::Left) {
                attack.attacking = true; //set attacking to true to override movement animations

                // deciding initial frame for swing (so not partial animation)
                if abx > aby {
                    if v.velocity.x >= 0. {
                        ratlas.index = 8;
                    } else if v.velocity.x < 0. {
                        ratlas.index = 12;
                    }
                } else {
                    if v.velocity.y >= 0. {
                        ratlas.index = 0;
                    } else if v.velocity.y < 0. {
                        ratlas.index = 4;
                    }
                }
                /* increment carnage. stupid fer now */
                if carnage.carnage < 50. {
                    carnage.carnage += 1.;
                }
                timer.reset();
            }
            if attack.attacking == true {
                timer.tick(time.delta());

                if abx > aby {
                    if v.velocity.x >= 0. {
                        if timer.finished() {
                            ratlas.index = ((ratlas.index + 1) % 4) + 8;
                        }
                        if ratlas.index == 11 {
                            attack.attacking = false;
                            ratlas.index = 24
                        } //allow for movement anims after last swing frame
                    } else if v.velocity.x < 0. {
                        if timer.finished() {
                            ratlas.index = ((ratlas.index + 1) % 4) + 12;
                        }
                        if ratlas.index == 15 {
                            attack.attacking = false;
                            ratlas.index = 28
                        } //allow for movement anims after last swing frame
                    }
                } else {
                    if v.velocity.y >= 0. {
                        if timer.finished() {
                            ratlas.index = (ratlas.index + 1) % 4;
                        }
                        if ratlas.index == 3 {
                            attack.attacking = false;
                            ratlas.index = 16
                        } //allow for movement anims after last swing frame
                    } else if v.velocity.y < 0. {
                        if timer.finished() {
                            ratlas.index = ((ratlas.index + 1) % 4) + 4;
                        }
                        if ratlas.index == 7 {
                            attack.attacking = false;
                            ratlas.index = 20
                        } //allow for movement anims after last swing frame
                    }
                }
            }
        }
    }
}

pub fn player_attack_enemy(
    mut commands: Commands,
    mut player: Query<(&Transform, &mut Attack, &NetworkId), With<Player>>,
    enemies: Query<(Entity, &mut Transform), (With<Enemy>, Without<Player>)>,
    client_id: Res<ClientId>,
) {
    for (ptransform, pattack, id) in player.iter_mut() {
        if id.id == client_id.id {
            if pattack.attacking == false {
                return;
            }
            let player_aabb =
                collision::Aabb::new(ptransform.translation, Vec2::splat((TILE_SIZE as f32) * 3.));

            for (ent, enemy_transform) in enemies.iter() {
                let enemy_aabb =
                    Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
                if player_aabb.intersects(&enemy_aabb) {
                    commands.entity(ent).despawn();
                }
            }
        }
    }
}

// /* Spawns in user player, uses PlayerBundle for some consistency*/
// pub fn client_spawn_user_player(
//     commands: &mut Commands,
//     asset_server: &Res<AssetServer>,
//     texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
//     _id: u8,
// ) {
//     let player_sheet_handle = asset_server.load("player/4x12_player.png");
//     let player_layout = TextureAtlasLayout::from_grid(
//         UVec2::splat(TILE_SIZE),
//         PLAYER_SPRITE_COL,
//         PLAYER_SPRITE_ROW,
//         None,
//         None,
//     );
//     let player_layout_len = player_layout.textures.len();
//     let player_layout_handle = texture_atlases.add(player_layout);

//     // spawn player at origin
//     commands.spawn(ClientPlayerBundle {
//         sprite: SpriteBundle {
//             texture: player_sheet_handle,
//             transform: Transform::from_xyz(0., 0., 900.),
//             ..default()
//         },
//         atlas: TextureAtlas {
//             layout: player_layout_handle,
//             index: 0,
//         },
//         animation_timer: AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
//         animation_frames: AnimationFrameCount(player_layout_len),
//         velo: Velocity::new(),
//         id:NetworkId::new(),
//         player: Player,
//         health: Health::new(),
//         crouching: Crouch{crouching:false},
//         rolling: Roll{rolling:false},
//         sprinting: Sprint{sprinting:false},
//         attacking: Attack{attacking:false}
//     });
// }

/* deprecated */
pub fn client_spawn_other_player_new(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    player: PlayerSendable,
    source_ip: SocketAddr,
) {
    let player_sheet_handle = asset_server.load("player/4x8_player.png");
    let player_layout = TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
        PLAYER_SPRITE_COL,
        PLAYER_SPRITE_ROW,
        None,
        None,
    );
    let player_layout_len = player_layout.textures.len();
    let player_layout_handle = texture_atlases.add(player_layout);
    // spawn player at origin
    commands.spawn(ClientPlayerBundle {
        sprite: SpriteBundle {
            texture: player_sheet_handle,
            transform: player.transform,
            ..default()
        },
        rolling: Roll {
            rolling: player.roll,
        },
        atlas: TextureAtlas {
            layout: player_layout_handle,
            index: 0,
        },
        animation_timer: AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
        animation_frames: AnimationFrameCount(player_layout_len),
        velo: Velocity {
            velocity: player.velocity,
        },
        id: NetworkId {
            id: player.head.network_id,
            addr: source_ip,
        },
        player: Player,
        health: player.health,
        crouching: Crouch {
            crouching: player.crouch,
        },
        sprinting: Sprint {
            sprinting: player.sprint,
        },
        attacking: Attack {
            attacking: player.attack,
        },
        inputs: InputQueue::new(),
        states: PastStateQueue::new()
    });
}

/*deprecated */
pub fn client_spawn_other_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    player: PlayerSendable,
    source_ip: SocketAddr,
) {
    let player_sheet_handle = asset_server.load("player/4x8_player.png");
    let player_layout = TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
        PLAYER_SPRITE_COL,
        PLAYER_SPRITE_ROW,
        None,
        None,
    );
    let player_layout_len = player_layout.textures.len();
    let player_layout_handle = texture_atlases.add(player_layout);

    // spawn player at origin
    commands.spawn(ClientPlayerBundle {
        sprite: SpriteBundle {
            texture: player_sheet_handle,
            transform: Transform::from_xyz(
                player.transform.translation.x,
                player.transform.translation.y,
                900.,
            ),
            ..default()
        },
        rolling: Roll::new(),
        atlas: TextureAtlas {
            layout: player_layout_handle,
            index: 0,
        },
        animation_timer: AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
        animation_frames: AnimationFrameCount(player_layout_len),
        velo: Velocity::new(),
        id: NetworkId {
            id: player.head.network_id,
            addr: source_ip,
        },
        player: Player,
        health: Health::new(),
        crouching: Crouch { crouching: false },
        sprinting: Sprint { sprinting: false },
        attacking: Attack { attacking: false },
        inputs: InputQueue::new(),
        states: PastStateQueue::new()
    });
}

/* Checks for player interacting with game world.
 * E for interact? Assumed menu etc. could also
 * fit in here.. I also currently have pot as
 * it's own resource, maybe make an 'interactable'
 * trait for query? - rorto */
pub fn player_interact(
    mut player: Query<
        (&mut Transform, &mut Velocity, &NetworkId),
        (With<Player>, Without<Background>),
    >,
    input: Res<ButtonInput<KeyCode>>,
    client_id: Res<ClientId>,
    mut pot_q: Query<&mut Pot>,
    mut pot_transform_q: Query<&mut Transform, (With<Pot>, Without<Player>)>,
    mut texture_atlas: Query<&mut TextureAtlas, (With<Pot>, Without<Player>)>,
) {
    let mut pot = pot_q.single_mut();
    let pot_transform = pot_transform_q.single_mut();
    let mut pot_atlas = texture_atlas.single_mut();
    for (player_transform, mut _player_velocity, id) in player.iter_mut() {
        if id.id == client_id.id {
            /* Has nothing to do with particles */
            let pot_particle_collider =
                Aabb::new(pot_transform.translation, Vec2::splat(TILE_SIZE as f32));
            let player_particle_collider =
                collision::Aabb::new(player_transform.translation, Vec2::splat(TILE_SIZE as f32));

            /* touch is how many frames since pressed
             * We only want to increment if not pressed
             * recently */
            if input.just_pressed(KeyCode::KeyE)
                && pot_particle_collider.intersects(&player_particle_collider)
                && pot.touch == 0
            {
                info!("you got touched");
                pot.touch += 1;

                if pot.touch == 1 {
                    pot_atlas.index = pot_atlas.index + 1;
                }
                //TODO
            }
        }
    }
}

pub fn player_input(
    mut player: Query<
        (
            &mut Velocity,
            &mut Crouch,
            &mut Roll,
            &mut Sprint,
            &NetworkId,
        ),
        (With<Player>, Without<Background>),
    >,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    client_id: Res<ClientId>,
) {
    for (mut player_velocity, mut crouch_query, mut roll_query, mut sprint_query, player_id) in
        player.iter_mut()
    {
        if player_id.id == client_id.id {
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
            let crouch_multiplier = if input.pressed(KeyCode::KeyC) {
                crouch.crouching = true;
                CROUCH_MULTIPLIER
            } else {
                crouch.crouching = false;
                1.0
            };

            /* check if rolling */
            let roll = roll_query.as_mut();
            if input.pressed(KeyCode::KeyR) {
                roll.rolling = true;
            }

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
            if input.pressed(KeyCode::KeyA) {
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

            if adjusted_speed > current_max {
                /* here we are. moving too fast. Honestly, I don't
                 * think that we should clamp, we may have just crouched.
                 * We should decelerate by a given rate, our acceleration rate! s
                 * not using the adjusted, dont want if crouch slow slowdown yk */
                adjusted_speed -= ACCELERATION_RATE;
                let _boo = deltav.clamp_length_max(adjusted_speed);
            }

            /* final set */
            player_velocity.velocity = deltav;
        }
    }
}

/* Old setup had too much in one function, should collision check be
 * done in here??? */
pub fn update_player_position(
    time: Res<Time>,
    mut players: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
) {
    /* We use delta time to determine ur velocity earlier, so we really want to use it again here?
     * It gives second since update, not since we got input... */
    for (mut transform, velocity) in players.iter_mut() {
        transform.translation.x += velocity.velocity.x * time.delta_seconds();
        transform.translation.y += velocity.velocity.y * time.delta_seconds();

        let mut _hit_door = false;
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
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &NetworkId),
        (With<Player>, Without<Background>, Without<Door>),
    >,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>, Without<Door>)>,
    door_query: Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,
    mut room_manager: ResMut<RoomManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    room_query: Query<Entity, With<Room>>,
    client_id: Res<ClientId>,
    mut carnage: Query<&mut CarnageBar>,
) {
    let mut hit_door = false;
    let mut _player_transform = Vec3::ZERO;
    let mut door_type: Option<DoorType> = Option::None;

    // Player movement
    for (mut pt, mut pv, id) in player_query.iter_mut() {
        if id.id != client_id.id {
            continue;
        }
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

        let crouch_multiplier = if input.pressed(KeyCode::KeyC) {
            CROUCH_MULTIPLIER
        } else {
            1.0
        };

        // set new max speed
        let max_speed = PLAYER_SPEED * speed_multiplier * crouch_multiplier;

        /* check if rolling */
        /*let roll = roll_query.as_mut();
        if input.pressed(KeyCode::KeyR){
            roll.rolling = true;
        }*/

        pv.velocity = if deltav.length() > 0. {
            (pv.velocity + (deltav.normalize_or_zero() * acc)).clamp_length_max(max_speed)
        } else if pv.velocity.length() > acc {
            pv.velocity + (pv.velocity.normalize_or_zero() * -acc)
        } else {
            Vec2::splat(0.)
        };

        let change = pv.velocity * deltat;
        let (room_width, room_height) = room_manager.current_room_size();

        // let mut help = false;
        // if !help{
        //     //println!("--HELP-- Room Width: {} Room Height: {}",room_width,room_height);
        //     help = true;
        // }

        // Calculate new player position and clamp within room boundaries
        let new_pos_x = (pt.translation.x + change.x).clamp(
            -room_width / 2.0 + TILE_SIZE as f32 + TILE_SIZE as f32 / 2.0,
            room_width / 2.0 - TILE_SIZE as f32 - TILE_SIZE as f32 / 2.0,
        );
        let new_pos_y = (pt.translation.y + change.y).clamp(
            -room_height / 2.0 + TILE_SIZE as f32 + TILE_SIZE as f32 / 2.0,
            room_height / 2.0 - TILE_SIZE as f32 - (TILE_SIZE / 2) as f32 / 2.0,
        );

        pt.translation.x = new_pos_x;
        pt.translation.y = new_pos_y;

        // Store the player's position for later use
        _player_transform = pt.translation;

        let baban = handle_movement_and_enemy_collisions(
            &mut pt,
            change,
            &mut enemies,
            &mut room_manager,
            &door_query,
        );
        hit_door = baban.0;
        door_type = baban.1;
    }
    // If a door was hit, handle the transition
    if hit_door {
        let mut carnage_bar = carnage.single_mut();
        carnage_bar.stealth += 10.;
        if let Some(door_type) = door_type {
            // Pass the door type to transition_map
            transition_map(
                &mut commands,
                &asset_server,
                &mut room_manager,
                room_query,
                door_query,
                &mut player_query.single_mut().0,// this is broke cant be single
                door_type,
                carnage,
            );
        }
    }
}

pub fn handle_movement_and_enemy_collisions(
    pt: &mut Transform,
    change: Vec2,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>, Without<Door>)>,
    room_manager: &mut RoomManager,
    door_query: &Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,
) -> (bool, Option<DoorType>) {
    let mut hit_door = false;
    let mut door_type = None;

    // Calculate new player position
    let new_pos = pt.translation + Vec3::new(change.x, change.y, 0.0);
    let player_aabb = collision::Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    // Translate player position to grid indices
    let (_topleft, _topright, _bottomleft, _bottomright) =
        translate_coords_to_grid(&player_aabb, room_manager);

    // Translate player position to grid indices
    let _grid_x = (new_pos.x / TILE_SIZE as f32).floor();
    let _grid_y = (new_pos.y / TILE_SIZE as f32).floor();
    //println!("Player grid position: x = {}, y = {}", grid_x, grid_y);

    // Handle collisions and movement within the grid
    handle_movement(
        pt,
        Vec3::new(change.x, 0., 0.),
        room_manager,
        enemies,
        &door_query,
    );
    handle_movement(
        pt,
        Vec3::new(0., change.y, 0.),
        room_manager,
        enemies,
        &door_query,
    );

    for (door_transform, door) in door_query.iter() {
        let door_aabb = Aabb::new(door_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if player_aabb.intersects(&door_aabb) {
            hit_door = true;
            door_type = Some(door.door_type);
            break;
        }
    }

    // Return the hit_door state and door type
    (hit_door, door_type)
}

pub fn handle_movement(
    pt: &mut Transform,
    change: Vec3,
    room_manager: &mut RoomManager,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>, Without<Door>)>,
    door_query: &Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,
) -> Option<DoorType> {
    let new_pos = pt.translation + change;
    let player_aabb = collision::Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    // Get the current room's grid size (room width and height)
    let current_grid = room_manager.current_grid();
    let room_width = current_grid.len() as f32 * TILE_SIZE as f32;
    let room_height = current_grid[0].len() as f32 * TILE_SIZE as f32;

    let (topleft, topright, bottomleft, bottomright) =
        translate_coords_to_grid(&player_aabb, room_manager);

    // check for collisions with enemies
    for enemy_transform in enemies.iter() {
        let enemy_aabb = Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if player_aabb.intersects(&enemy_aabb) {
            // handle enemy collision here (if necessary)
            return None;
        }
    }

    // movement within bounds and wall/door collision check
    if new_pos.x >= -room_width / 2.0 + TILE_SIZE as f32 / 2.
        && new_pos.x <= room_width / 2.0 - TILE_SIZE as f32 / 2.
        && new_pos.y >= -room_height / 2.0 + TILE_SIZE as f32 / 2.
        && new_pos.y <= room_height / 2.0 - TILE_SIZE as f32 / 2.
        && topleft != 1
        && topright != 1
        && bottomleft != 1
        && bottomright != 1
    {
        pt.translation = new_pos;
    }

    // check for door transition
    if topleft == 2 || topright == 2 || bottomleft == 2 || bottomright == 2 {
        // If door is hit, return the door type
        for (door_transform, door) in door_query.iter() {
            let door_aabb = Aabb::new(door_transform.translation, Vec2::splat(TILE_SIZE as f32));
            if player_aabb.intersects(&door_aabb) {
                return Some(door.door_type); // Return the type of door hit
            }
        }
    }

    None // No door was hit
}

pub fn animate_player(
    time: Res<Time>,
    mut player: Query<
        (
            &Velocity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &AnimationFrameCount,
            &Attack,
            &Roll,
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
    for (v, mut ratlas, mut timer, _frame_count, attack, roll) in player.iter_mut() {
        if attack.attacking == true {
            return;
        } //checking if attack animations are running
        if roll.rolling == true {
            return;
        } //checking if roll animations are running
        timer.tick(time.delta());

        let abx = v.velocity.x.abs();
        let aby = v.velocity.y.abs();

        if abx > aby {
            if v.velocity.x > 0. {
                if timer.finished() {
                    ratlas.index = ((ratlas.index + 1) % 4) + 24;
                }
            } else if v.velocity.x < 0. {
                if timer.finished() {
                    ratlas.index = ((ratlas.index + 1) % 4) + 28;
                }
            }
        } else {
            if v.velocity.y > 0. {
                if timer.finished() {
                    ratlas.index = ((ratlas.index + 1) % 4) + 16;
                }
            } else if v.velocity.y < 0. {
                if timer.finished() {
                    ratlas.index = ((ratlas.index + 1) % 4) + 20;
                }
            }
        }
    }
}

pub fn player_roll(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<
        (
            &Velocity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &AnimationFrameCount,
            &Attack,
            &mut Roll,
            &NetworkId,
        ),
        With<Player>,
    >,
    client_id: Res<ClientId>,
) {
    /* In texture atlas for ratatta:
     * 36 - 39 = up
     * 32- 35 = down
     * 44 - 47 = right
     * 40 - 43 = left
     * ratlas. heh. get it.*/
    for (v, mut ratlas, mut timer, _frame_count, attack, mut roll, id) in player.iter_mut() {
        if id.id == client_id.id {
            let abx = v.velocity.x.abs();
            let aby = v.velocity.y.abs();

            if attack.attacking == true {
                return;
            } //do not roll if swinging

            if input.pressed(KeyCode::KeyR) {
                roll.rolling = true;
                if abx > aby {
                    if v.velocity.x >= 0. {
                        ratlas.index = 44;
                    } else if v.velocity.x < 0. {
                        ratlas.index = 40;
                    }
                } else {
                    if v.velocity.y >= 0. {
                        ratlas.index = 36;
                    } else if v.velocity.y < 0. {
                        ratlas.index = 32;
                    }
                }
                timer.reset();
            }

            if roll.rolling == true {
                timer.tick(time.delta());

                if abx > aby {
                    if v.velocity.x >= 0. {
                        if timer.finished() {
                            ratlas.index = ((ratlas.index + 1) % 4) + 44;
                        }
                        if ratlas.index == 47 {
                            roll.rolling = false;
                            ratlas.index = 24
                        } //allow for movement anims after last swing frame
                    } else if v.velocity.x < 0. {
                        if timer.finished() {
                            ratlas.index = ((ratlas.index + 1) % 4) + 40;
                        }
                        if ratlas.index == 43 {
                            roll.rolling = false;
                            ratlas.index = 28
                        } //allow for movement anims after last swing frame
                    }
                } else {
                    if v.velocity.y >= 0. {
                        if timer.finished() {
                            ratlas.index = ((ratlas.index + 1) % 4) + 36;
                        }
                        if ratlas.index == 39 {
                            roll.rolling = false;
                            ratlas.index = 16
                        } //allow for movement anims after last swing frame
                    } else if v.velocity.y < 0. {
                        if timer.finished() {
                            ratlas.index = ((ratlas.index + 1) % 4) + 32;
                        }
                        if ratlas.index == 35 {
                            roll.rolling = false;
                            ratlas.index = 20
                        } //allow for movement anims after last swing frame
                    }
                }
            }
        }
    }
}
