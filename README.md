# Cuscuta

by Cuscuta

## Team Members
* Advanced Topic Subteam 1: Networked Multiplayer
	* Chase Lahner
	* Rory McCann
	* Lukas Finn

* Advanced Topic Subteam 2: Procedural Generation
	* Tyler Ryan
	* Nico Zeuss

## Game Description

Cuscuta is a biopunk rougelike dungeon-crawler, with emphasis on stealth mechanics and intuitive world generation


## Advanced Topic Description

### Networked Multiplayer

 + Multiplayer will function using a P2P connection between the two players
 + Each player has a unique carnage bar allowing for more build combinations
 + Procedural rooms are generated based off an average of both players' carnage bars, weighted heavier towards carnage
 + Teammates must balance between stealth and destruction to maintain a balance in the dangerous dungeon
 + Dungeons will be harder in co-op mode and therefore require strategy and planning


    
### Procedural Generation
+ Room size: Room size is variable and will increase/decrease depending on the carnage bar
+ Enemies: Enemy count and type is variable based on the carnage bar
+ Obstacles: Obstacles & stealth objects will spawn differently depending on the carnage bar
+ Replayable - Each playthrough is entirely procedurally generated, making it feel like a new run each time
+ Variable - Automatic balancing between 1-2 players, 2 player runs will have more enemies/difficulty
	##### Method
	1. Create (numRooms) rooms with different shapes
	2. Push the rooms apart until they are not touching
	3. Determine "main" rooms (boss rooms, starting room)
	4. Delauney triangulate the main rooms
	5. Create a MST from the graph to create edges and loops
	6. Add hallways

## Midterm Goals
* Playable Demo
	+ at least 1 enemy type with basic movement (basic, non-player oriented) and causes damage when colliding with a player
	+ 2D movement - Up, down, left, right
	+ Crouch/sneak mechanic, dash mechanic implemented
* Generation
	+ Basic dungeon generation - fluctuating room size & metrics, completely random dungeon generation
	+ Entire dungeon generation at runtime
* Rudimentary Multiplayer
	+ Basic two-player connection - Not lag free, just able to have 2 players in same game
* Rudimentary Animation/Artwork
   	* Starter sword art
	* Character art & animations
   	* 2 enemy sprites & animations   	
* Basic Line-of-Sight Detection for 1 enemy type
  	+ move enemy to player's last known location (LKL)

## Final Goals
*  Complete Game
	+ Dungeon generation based on players' carnage bars and location - Should be random, but influenced by carnage bars and player count
		+ Too high carnage will generate rooms with difficult opponents to fight, but easy to sneak around
		+ Too low carnage will generate rooms with easier enemies to fight, but too many to sneak around
	+ Tight interplay between generation & multiplayer - 2 players mean more difficult rooms
 		+ More enemies will be spawned in with 2 players
		+ Some rooms require teamwork to complete
	+ At least 4 enemy types
 	+ One distinct item players can utilize
   	+ Ability to escape detection from enemies
* Finalized Generation
	+ Each room is generated realtime based on the carnage bar
	+ Rooms generated as you step into them, not all at the beginning of the game
	+ Enemy count & type fluctuates as they spawn real-time
	+ Dungeon "fights against" you
*  Fluid Multiplayer
	+ Movement and monsters must be synchronized 
	+ No large room lag
	+ Movement must be relatively lag-free
*  Ending
	+  End reward

## Stretch Goals

* Advanced Stealth/Detection System
	+ Ability to distract enemy
	+ Enemies pathfind to player's last known location
* At least 3-4 distinct items player can utilize (At least 2 weapons, 1 buff)
	+ Starter sword, gun, bow, knife, potions/powerup, etc
* Boss Battle
	*  Boss has at least 2 unique abilities, special reward for winning

## Grade Breakdown (WIP)

* Dungeon procedural generation works as intended: 25%
	* Variable generation based on carnage bar: 10%
	* Completely Random generation -- rooms from start to final boss: 10%
	* Variable generation/difficulty based on 1-2 players: 5%
* Multiplayer connection works as intended: 25%
	* Basic functionality: 2 players with full functionality in same lobby: 12.5%
	* Lack of major lag/latency in basic game functions: 12.5%
* Automatic playercount-dependent and carnage-bar-dependent game balancing: 15%
* Fluid movement mechanics (Walk, Sprint, Roll) -> can be used in succession: 7.5%
* Final boss battle -> At least 2 unique enemy abilities: 7.5%
* Minimum Viable Product (all goals reached) 10%
* Working stealth/detection system: 5%
* Additional playable character: 5%

## CHANGES (DELETE WHEN ALL DONE)
P2P networking or dedicated server? - DONE, P2P
Midterm goals
	A bit more specifics on enemy type (e.g., does it attack? movement only? what should it be able to do?) - DONE
	Static placeholder dungeon map OK for midterm
Final goals / stretch goals
	Specifics on how the room generation is based on carnage bars - DONE
	How should the difficulty scale with 2 players? - DONE
	How should the enemy types differ? What should each be able to do?
	Cut to 1 item, moved other items to stretch goal - DONE
	Cut boss battle or move to stretch goal - DONE
	Move ability to escape detection to final goal - DONE
	Stretch goals should be pick 2 of:
		Enemy distraction and pathing towards player's last known location
		Boss battle
		3 more item types
	Cut Multiple characters - DONE
 Grading
 	Cut Multiple characters
	Again, clarify carnage effect on ProcGen and 2 player scaling
	Remove playercount-dependent and carnage-bar-dependent game balancing goal, already included in ProcGen goal, add points for detection/escape, and increase points for basic movement
	Cut boss battle, add points for enemy AI working (very basic ai as that's not an adv topic)
