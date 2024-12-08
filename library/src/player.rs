use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::enemies::{EnemyId, EnemyToKill};
use crate::network::{ClientPacket, DecreaseEnemyHealthPacket, Header, KillEnemyPacket, Sequence, UDP};

use crate::{
    collision::{self, *},
    cuscuta_resources::*,
    enemies::Enemy,
    network::PlayerSendable,
    room_gen::*,
    ui::CarnageBar,
    markov_chains::LastAttributeArray,
};

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Player; // wow! it is he!

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Trackable; //used for enemy pathfinding

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Monkey;

/* used by monkey to kill after x time */
#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Lifetime {
    pub life: u32,
}
impl Lifetime {
    pub fn new() -> Self {
        Self { life: 0 }
    }
}

#[derive(Bundle)]
pub struct ClientCymbalMonkey {
    pub track: Trackable,
    pub sprite: SpriteBundle,
    pub distracto: Monkey,
    pub atlas: TextureAtlas,
    pub animation_timer: AnimationTimer,
    pub doom_timer: DoomTimer,
    pub animation_frames: AnimationFrameCount,
    pub lifetime: Lifetime,
}

#[derive(Bundle)]
pub struct ServerCymbalMonkey {
    pub track: Trackable,
    pub monke: Monkey,
    pub lifetime: Lifetime,
    pub transform: Transform,
}

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
    pub fn new_set(b: bool) -> Self {
        Self { crouching: b }
    }
    pub fn set(&mut self, crouch: bool) {
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
    pub fn new_set(b: bool) -> Self {
        Self { rolling: b }
    }
    pub fn set(&mut self, roll: bool) {
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
    pub fn new_set(b: bool) -> Self {
        Self { sprinting: b }
    }
    pub fn set(&mut self, sprint: bool) {
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
    pub fn new_set(b: bool) -> Self {
        Self { attacking: b }
    }
    pub fn set(&mut self, att: bool) {
        self.attacking = att;
    }
}

#[derive(Resource)]
pub struct CollisionState {
    pub colliding_with_wall: bool,
    pub last_position: Vec3,
}

impl CollisionState {
    pub fn new() -> Self {
        Self {
            colliding_with_wall: false,
            last_position: Vec3::ZERO,
        }
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

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct ItemStatus {
    pub has_potion: bool,
    pub has_monkey: bool,
}

impl ItemStatus {
    pub fn new() -> Self {
        Self {
            has_monkey: false,
            has_potion: false,
        }
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
    pub states: PastStateQueue,
    pub potion_status: ItemStatus,
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
    pub player: Player,
    pub track: Trackable, //pub inputs: InputQueue,
                          //pub time: Timestamp,
}

#[derive(Component)]
pub struct EnemyPastStateQueue {
    pub q: VecDeque<EnemyPastState>,
}

impl EnemyPastStateQueue {
    pub fn new() -> Self {
        Self {
            q: VecDeque::with_capacity(2),
        }
    }
}

pub struct EnemyPastState {
    pub transform: Transform,
}

#[derive(Component, Serialize, Deserialize)]
pub struct PastStateQueue {
    pub q: VecDeque<PastState>, // double ended queue, will wrap around when full
}

impl PastStateQueue {
    pub fn new() -> Self {
        Self {
            q: VecDeque::with_capacity(2), // store current and previous states
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PastState {
    pub velo: Velocity,
    pub transform: Transform,
    pub crouch: Crouch,
    pub roll: Roll,
    pub attack: Attack,
    pub seq: Sequence,
}

impl PastState {
    pub fn new() -> Self {
        Self {
            velo: Velocity::new(),
            transform: Transform::from_xyz(0., 0., 0.),
            crouch: Crouch::new(),
            roll: Roll::new(),
            attack: Attack::new(),
            seq: Sequence::new(0),
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
    mut enemies: Query<(Entity, &mut Transform, &mut EnemyId, &mut Health), (With<Enemy>, Without<Player>)>,
    client_id: Res<ClientId>,
    udp: Res<UDP>,
    sequence: ResMut<Sequence>
) {
   // info!("checking for player attack");
    for (ptransform, pattack, id) in player.iter_mut() {
        if id.id == client_id.id {
            if pattack.attacking == false {
                return;
            }
          //  info!("attta    cking");
            let player_aabb =
                collision::Aabb::new(ptransform.translation, Vec2::splat((TILE_SIZE as f32) * 3.));

            for (ent, enemy_transform, id, mut enemy_health) in enemies.iter_mut() {
               // info!("enemy has health or whateva");
                let enemy_aabb =
                    Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
                if player_aabb.intersects(&enemy_aabb) {
                    enemy_health.current -= 1.;
                    info!("enemy health: {}", enemy_health.current);
                    let packet = ClientPacket::DecreaseEnemyHealthPacket(DecreaseEnemyHealthPacket  {
                        enemy_id: id.clone(),
                        decrease_by: 1.,
                    });
                    let mut serializer = flexbuffers::FlexbufferSerializer::new();
                    packet.serialize(&mut serializer).unwrap();
                    let packet: &[u8] = serializer.view();
                    udp.socket.send_to(&packet, SERVER_ADR).unwrap();

                    if enemy_health.current <= 0. {
                        let packet = ClientPacket::KillEnemyPacket(KillEnemyPacket {
                            enemy_id: id.clone(),
                        });
                        let mut serializer = flexbuffers::FlexbufferSerializer::new();
                        packet.serialize(&mut serializer).unwrap();
                        let packet: &[u8] = serializer.view();
                        info!("Sending packet to kill enemy");
                        udp.socket.send_to(&packet, SERVER_ADR).unwrap();
                        commands.entity(ent).despawn();
                    }    
                }
            }
        }
    }
}

pub fn check_handle_player_death(
    mut commands: Commands,
    us: Res<ClientId>,
    mut players: Query<(Entity, &Health, &NetworkId, &mut Visibility), With<Player>>,
    udp: Res<UDP>,
    mut death_timer: ResMut<PlayerDeathTimer>,
    time: Res<Time>,
) {
    for (entity, health, id, mut visibility) in players.iter_mut() {
        if us.id != id.id {
            continue;
        }
        // we have found us!
        if health.current <= 0. {
            // info!("we r dead");
            // we are dead
            *visibility = Visibility::Hidden;
            // *visibility = Visibility::Visible; // respawn player

            // despawn player, wait 5 secs, respawn player
        }
    }
}

pub fn tick_timer(
    mut commands: Commands,
    us: Res<ClientId>,
    mut players: Query<(Entity, &mut Health, &NetworkId, &mut Visibility), With<Player>>,
    udp: Res<UDP>,
    mut death_timer: ResMut<PlayerDeathTimer>,
    time: Res<Time>,
) {

    for (entity, mut health, id, mut visibility) in players.iter_mut() {
        //info!("cur health: {}", health.current);
        if us.id != id.id {
            continue;
        }
        if health.current <= 0. {
            // info!("ticking");
            death_timer.timer.tick(time.delta());
        }

        if death_timer.timer.finished() {
            info!("respawning");
            *visibility = Visibility::Visible;
            health.current = health.max;
            death_timer.timer.reset();
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
        states: PastStateQueue::new(),
        potion_status: ItemStatus::new(),
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
        health: Health::new_init(),
        crouching: Crouch { crouching: false },
        sprinting: Sprint { sprinting: false },
        attacking: Attack { attacking: false },
        inputs: InputQueue::new(),
        states: PastStateQueue::new(),
        potion_status: ItemStatus::new(),
    });
}

/* Checks for player interacting with game world.
 * E for interact? Assumed menu etc. could also
 * fit in here.. I also currently have pot as
 * it's own resource, maybe make an 'interactable'
 * trait for query? - rorto */
pub fn player_interact(
    mut commands: Commands,
    mut player: Query<
        (&mut Transform, &mut Velocity, &NetworkId, &mut ItemStatus),
        (With<Player>, Without<Background>),
    >,
    input: Res<ButtonInput<KeyCode>>,
    client_id: Res<ClientId>,
    mut pot_q: Query<(& Transform, &mut Pot, &mut TextureAtlas), (With<Pot>, Without<Player>)>,
    potion_query: Query<(Entity, &Transform), (With<Potion>, Without<Player>, Without<Pot>)>,
) {
    for (player_transform, mut _player_velocity, id, mut potion_status) in player.iter_mut() {
        if id.id == client_id.id {
            // player collider
            let player_collider =
                collision::Aabb::new(player_transform.translation, Vec2::splat(TILE_SIZE as f32));
            // loop through potions in room
            for (potion_entity, potion_transform) in potion_query.iter() {
                let potion_collider =
                    Aabb::new(potion_transform.translation, Vec2::splat(TILE_SIZE as f32));

                // if player intersects
                if player_collider.intersects(&potion_collider) && !potion_status.has_potion {
                    // check here if player is already carrying potion
                    potion_status.has_potion = true; // Player now has a potion
                    info!(
                        "Player at {:?} picked up a potion at {:?}!",
                        player_transform.translation, potion_transform.translation
                    );

                    // despawn potion
                    commands.entity(potion_entity).despawn();
                }
            }

            for (pot_transform, mut pot, mut pot_atlas) in pot_q.iter_mut(){

                 // Coin pot collider
                let pot_collider =
                Aabb::new(
                    pot_transform.translation, 
                    Vec2::splat(TILE_SIZE as f32)
                );
                /* touch is how many frames since pressed
                * We only want to increment if not pressed
                * recently */
                if input.just_pressed(KeyCode::KeyE)
                    && pot_collider.intersects(&player_collider)
                    && pot.touch == 0
                {
                    info!("you got touched");
                    pot.touch += 1;

                    if pot.touch == 1 {
                        pot_atlas.index = pot_atlas.index + 1;
                    }
                
                }
            }
        }
    }
}

pub fn spawn_monkey(
    player_q: Query<(&Transform, &NetworkId), (With<Player>)>,
    mut command: Commands,
    us: Res<ClientId>,
    keys: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    udp: Res<UDP>,
) {
    /* for all players, we match to find us */
    for (t, id) in player_q.iter() {
        if us.id != id.id {
            continue;
        }
        /* can assume that we are us yk yk yk (client player nont p2) */

        // AAAA_______________AAAA______________________
        // VVVV               VVVV
        // (__)               (__)
        //  \ \               / /
        //   \ \   \\|||//   / /
        //    > \   _   _   / <
        //     > \ / \ / \ / <
        //      > \\_o_o_// <
        //       > ( (_) ) <
        //        >|     |<
        //       / |\___/| \
        //       / (_____) \
        //       /         \
        //        /   o   \
        //         ) ___ (
        //        / /   \ \
        //       ( /     \ )
        //       ><       ><
        //      ///\     /\\\
        //      '''       '''               Michel Boisset -- grabbed from some ascii art site
        /* MAKE DA MONKE */
        if keys.just_pressed(KeyCode::Tab) {
            /* lots of spawning in stuffs. textures are a lot */
            let monkey_handle: Handle<Image> = asset_server.load(MONKEY_HANDLE);
            let monkey_layout = TextureAtlasLayout::from_grid(
                UVec2::splat(TILE_SIZE),
                MONKEY_SPRITE_COL,
                MONKEY_SPRITE_ROW,
                None,
                None,
            );
            let monkey_len = monkey_layout.textures.len();
            let monkey_layout_handle = texture_atlases.add(monkey_layout);
            info!("Monkey spawn");
            command.spawn(ClientCymbalMonkey {
                track: Trackable,
                sprite: SpriteBundle {
                    texture: monkey_handle,
                    transform: t.clone(),
                    ..default()
                },
                distracto: Monkey,
                atlas: TextureAtlas {
                    layout: monkey_layout_handle,
                    index: 0,
                },
                animation_timer: AnimationTimer(Timer::from_seconds(
                    ANIM_TIME,
                    TimerMode::Repeating,
                )),
                doom_timer: DoomTimer(Timer::from_seconds(
                    5.0,
                    TimerMode::Repeating,
                )),
                animation_frames: AnimationFrameCount(monkey_len),
                lifetime: Lifetime::new(),
            });

            /* once we have spawned in, then we swtich off to send it to the server, so
             * the enemies can pathfind to it, and os that the other clients can also have a lil peek */
        }
    }
}

pub fn update_monkey(
    mut commands: Commands,
    time: Res<Time>,
    mut monke: Query<
        (
            Entity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &mut DoomTimer,
            &AnimationFrameCount,
        ),
        With<Monkey>,
    >,
) {
    for (entity, mut ratlas, mut timer,mut doom, _frame_count) in monke.iter_mut() {
        timer.tick(time.delta());
        doom.tick(time.delta());
        if timer.finished() {ratlas.index = (ratlas.index + 1) % 2;} 
        if doom.finished() {commands.entity(entity).despawn();}
    }
}

pub fn restore_health(
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Health, &mut ItemStatus), With<Player>>,
) {
    for (mut health, mut potion_status) in player.iter_mut() {
        // check if the player has a potion and presses H
        if input.just_pressed(KeyCode::KeyH)
            && potion_status.has_potion
            && health.current < health.max
        {
            // restore 50 health and clamp
            health.current = (health.current + 25.).min(health.max);

            // set has potion to false
            potion_status.has_potion = false;
        }

        // take damage keybind for testing
        if input.just_pressed(KeyCode::KeyV) {
            health.current = (health.current - 25.);
            info!("Taking damage from keybind!");
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
    }
}

/* hopefully deprecated soon ^^ new one ^^
 * lil too much going on here, must be broke down */
pub fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &NetworkId, &mut Health),
        (With<Player>, Without<Background>, Without<Door>),
    >,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>, Without<Door>)>,
    door_query: Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,
    mut room_manager: ResMut<ClientRoomManager>,
    client_id: Res<ClientId>,
    inner_wall_query: Query<
        (&Transform),
        (
            With<InnerWall>,
            Without<Player>,
            Without<Enemy>,
            Without<Door>,
        ),
    >,
    mut collision_state: ResMut<CollisionState>,
) {
    
    let mut hit_door = false;
    let mut _player_transform = Vec3::ZERO;
    let mut door_type: Option<DoorType> = Option::None;

    // Player movement
    for (mut pt, mut pv, id, health) in player_query.iter_mut() {
        if id.id != client_id.id {
            continue;
        }
        if health.current <= 0. {
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
        let room_width = room_manager.width;
        let room_height = room_manager.height;

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

        if !collision_state.colliding_with_wall {
            collision_state.last_position = pt.translation;
        } else {
            pt.translation = collision_state.last_position;
        }

        handle_player_collisions(
            &mut pt,
            change,
            &mut enemies,
            &mut room_manager,
            &door_query,
            &inner_wall_query,
            &mut collision_state,
        );
    }
}


pub fn handle_player_collisions(
    pt: &mut Transform,
    change: Vec2,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>, Without<Door>)>,
    room_manager: &mut ClientRoomManager,
    door_query: &Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,
    inner_wall_query: &Query<
        (&Transform),
        (
            With<InnerWall>,
            Without<Player>,
            Without<Enemy>,
            Without<Door>,
        ),
    >,
    mut collision_state: &mut ResMut<CollisionState>,
) -> (bool, Option<DoorType>) {
    let mut hit_door = false;
    let mut door_type = None;

    // Calculate new player position
    let new_pos = pt.translation + Vec3::new(change.x, change.y, 0.0);
    let player_aabb = collision::Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    collision_state.colliding_with_wall = false;

    // Get the current room's grid size (room width and height)
    let room_height = room_manager.height;
    let room_width = room_manager.width;

    // Check for collisions with enemies
    for enemy_transform in enemies.iter() {
        let enemy_aabb = Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if player_aabb.intersects(&enemy_aabb) {
            // Collision with an enemy
            return (false, None);
        }
    }

    // Check if movement is within bounds and avoid collisions with walls
    if new_pos.x >= -room_width / 2.0 + TILE_SIZE as f32 / 2.
        && new_pos.x <= room_width / 2.0 - TILE_SIZE as f32 / 2.
        && new_pos.y >= -room_height / 2.0 + TILE_SIZE as f32 / 2.
        && new_pos.y <= room_height / 2.0 - TILE_SIZE as f32 / 2.
    {
        pt.translation = new_pos;
    } else {
        pt.translation = pt.translation - Vec3::new(change.x, change.y, 0.0);
        collision_state.colliding_with_wall = true;
        return (false, None);
    }

    // Check for collisions with inner walls
    for inner_wall_transform in inner_wall_query.iter() {
        let inner_wall_aabb = Aabb::new(
            inner_wall_transform.translation,
            Vec2::splat(TILE_SIZE as f32),
        );
        if player_aabb.intersects(&inner_wall_aabb) {
            pt.translation = pt.translation - Vec3::new(change.x, change.y, 0.0);
            collision_state.colliding_with_wall = true;
            return (false, None);
        }
    }

    // Check for collisions with doors
    for (door_transform, door) in door_query.iter() {
        let door_aabb = Aabb::new(door_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if player_aabb.intersects(&door_aabb) {
            hit_door = true;
            door_type = Some(door.door_type);
            break;
        }
    }

    // Return whether a door was hit and the type of the door
    (hit_door, door_type)
}

pub fn check_door_collision(
    door_query: &Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,
    player_transform: &Transform,
)-> (bool, Option<DoorType>){
    let player_aabb = collision::Aabb::new(player_transform.translation,
         Vec2::splat(TILE_SIZE as f32));
    let mut hit_door = false;
    let mut door_type = None;
    for (door_transform, door) in door_query.iter() {
        let door_aabb = Aabb::new(door_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if player_aabb.intersects(&door_aabb) {
            hit_door = true;
            door_type = Some(door.door_type);
            break;
        }
    }
    (hit_door, door_type)
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
