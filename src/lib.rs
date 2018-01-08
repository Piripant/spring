#[macro_use]
extern crate gfx;
extern crate nalgebra;
extern crate piston_window;
extern crate imgui;

type Vector = nalgebra::Vector2<f64>;

pub mod physics;
pub mod viewer;
pub mod shapes;
pub mod collisions;
