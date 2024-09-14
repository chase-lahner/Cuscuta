# Cuscuta

## Overview
Cuscuta is a biopunk rougelike dungeon-crawler, with emphasis on stealth mechanics and intuitive world generation

## Core Mechanics / Advanced Topics
1. Carnage Bar - Measure of how stealth oriented the player has been
2. **Procedural Generation** - Dynamically gernerated rooms with characteristics changing with carnage bar (Adv. Topic)
3. **Networked Multiplayer** - 2 player co-op with emphasis on working together (Adv. Topic)
4. Movement - Easy to learn, fluid mechanics that can be used in conjunction to create creative combinations

### Procedural Generation (Adv. Topic)
+ **Dynamic** - Based off the carnage bar, the type of room, enemies, obstacles, etc. will change
+ **Replayable** - Each playthrough is entirely procedurally generated, making it feel like a new run each time
+ **Variable** - Automatic balancing between 1-2 players, 2 player runs will have more enemies/difficulty

### Networked Multiplayer (Adv. Topic)
**Carnage**
 + Each player has a unique carnage bar allowing for more build combinations
 + Procedural rooms are generated based off an average of both players' carnage bars, weighted heavier towards carnage.

**Teamwork**
+ Teammates must balance between stealth and destruction to maintain abalance in the dangerous dungeon
+ dungeons will be harder in co-op mode and therefore require more strategy and planning

### Movement
+ Movement will have an emphasis on being simple, but precise, with core moves that flow seamlessly into one another. There will be a basic walk, sprint, and roll mechanic. Players will need to be able to utilize each movement mechanic to its full potential to sucessfully progress through the game. 

### Carnage Bar
+ Starts centralized, shifting based on how players tackle rooms towards either stealth or carnage. Towards carnage, the player becomes a glass cannon, with high damage but high risk of ending the run. Towards stealth, the player becomes weaker but stealthier, and it becomes much more difficult to fight impassable monsters. The goal of this is to incentivise switching up gameplay, keeping players on their toes

## Goals

### Midterm
+ **Playable Demo**
	+ Basic dungeon generation - fluctuating room size & metrics, completely random
	+ at least 1 enemy type
+ **Rudimentary Multiplayer**
	+ Basic two-player connection
	+ Basic synchronization between movement
+ **Rudimentary Animation/Artwork**
+ **Basic Line-of-Sight Detection**

### Final
+ **Complete Game**
	+ Dungeon generation based on players' carnage bars and location
	+ Tight interplay between generation & multiplayer
	+ At least 4 enemy types
+ **Fluid Multiplayer**
	+ Movement and monsters must be synchronized
	+ No large room lag
+ **Ending, Boss Battle**

### Stretch Goals
+ **Advanced Stealth/Detection system**
	+ Ability to escape detection
	+ Ability to distract enemy
	+ Enemies pathfind to player's last known location
+ **Multiple Characters**
	+ Two characters with at least one distinct ability each

