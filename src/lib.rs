#[macro_use]
extern crate gfx;
extern crate imgui;
extern crate nalgebra;
extern crate piston_window;

type Vector = nalgebra::Vector2<f64>;

pub mod physics;
pub mod viewer;
pub mod shapes;
