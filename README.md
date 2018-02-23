# Spring
Spring uses a mass-__spring__-damper system and continuos collision detection to archive 2D soft bodies simulations which are more sandbox/game like than accurate.

It is completely written in Rust, and uses as its main dependencies nalgebra for math and piston for the graphics.

## Features
* A world editor to dynamically add bodies and surfaces
* Continuos collision detection
* Fully configurable physical properties (mass, friction, damping ratio, joint strength)
