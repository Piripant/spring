extern crate nalgebra;
extern crate spring;

use nalgebra::Vector2;
use spring::{physics, shapes, viewer};

fn main() {
    let mut world = physics::simulation::World::new();
    shapes::make_polygon(&mut world, Vector2::new(7.0, 7.0), 5.0, 8);
    let view = viewer::ViewState::new(world);
    viewer::drawing::view_loop(view);
}
