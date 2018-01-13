use std::cell::RefCell;

use nalgebra::Vector2;
use physics::simulation::{Vertex, World};

pub fn make_polygon(world: &mut World, center: Vector2<f64>, radius: f64, num_verts: usize) {
    for i in 0..num_verts {
        use std::f64;

        let x = (i as f64 / num_verts as f64 * f64::consts::PI * 2.0).cos() * radius + center.x;
        let y = (i as f64 / num_verts as f64 * f64::consts::PI * 2.0).sin() * radius + center.y;

        world
            .verts
            .push(RefCell::new(Vertex::new(Vector2::new(x, y))));
    }

    world
        .verts
        .push(RefCell::new(Vertex::new(Vector2::new(center.x, center.y))));

    let len = world.verts.len();
    for i in 0..len {
        let mut next_index = (i + 1) % len;
        if i == len - 2 {
            next_index = 0;
        }

        world.create_joint(i, next_index);
        if len - 1 != i {
            world.create_joint(i, len - 1);
        }
    }
}
