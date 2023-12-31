# Arete Examples

A collection of demos and examples for [Arete Engine](https://github.com/aretegames/arete-engine).

If you just want to try out the demos, head over to the [releases page](https://github.com/aretegames/arete-examples/releases) for ready-to-run versions of the demos!

## Setup

To run from code, download the latest version of [Arete](https://github.com/aretegames/arete-engine/releases/tag/v0.1.0) and extract it so that your directory structure looks like:

- `arete-examples/`
    - `arete/` - from the engine zip
    - `arete-cpp/` - from the engine zip (**optional**, for C++ examples)
    - `arete-ios/` - from the engine zip (**optional**, for iOS)
    - `space-cpp/`
    - `tanks-cpp/`
    - `tanks-rust/`

You may now choose any project directory, such as `space-cpp/`, and execute the `run-*` script file to build and run the example!

## Examples

### Tanks

<img src="assets/tanks.jpg" height="320">

This demo is copied almost exactly from Unity's DOTS [tanks tutorial](https://github.com/Unity-Technologies/EntityComponentSystemSamples/blob/master/EntitiesSamples/Assets/Tutorials/Tanks/README.md). It is the simplest example to dive into, so we recommend starting here!

The demo simulates a number of tanks with random movement, each firing a cannonball once per frame. There is also a player-controlled tank, which is moved with WASD and fires when the spacebar is pressed.

### Space

<img src="assets/space.jpg" width="200">

Inspired by old Galaga/Space Invaders shoot'em up games crossed with the infamous Age of Origins ads, this demo is a more complete game example.

Enemies spawn in waves, moving down the screen towards the player. Waves steadily become more challenging, with more enemies and varied enemy types. The player may collect powerups which adds allies, each with a randomized laser or grenade-type weapon. There are also special powerups, which add health or wipe the entire screen of enemies.

The game area is designed to be played in portrait mode on a mobile device.
