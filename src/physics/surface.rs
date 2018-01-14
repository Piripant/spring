use std::cell::RefCell;
use physics::simulation::Vertex;
use Vector;

pub struct Surface {
    pub index_a: usize,
    pub index_b: usize,

    pub damping_ratio: f32,
    pub strength: f32,
    pub original_distance: f64,

    pub friction: f32,
    pub resitution: f32,
}

impl Surface {
    pub fn new(index_a: usize, index_b: usize, verts: &Vec<RefCell<Vertex>>) -> Surface {
        let vertex_a = verts[index_a].borrow();
        let vertex_b = verts[index_b].borrow();

        Surface {
            index_a,
            index_b,
            damping_ratio: 1.0,
            strength: 40.0,
            original_distance: (vertex_a.position - vertex_b.position).norm(),
            friction: 0.0,
            resitution: 1.0,
        }
    }

    pub fn apply_force(&self, verts: &Vec<RefCell<Vertex>>) {
        let mut vertex_a = verts[self.index_a].borrow_mut();
        let mut vertex_b = verts[self.index_b].borrow_mut();

        let mut force = Vector::new(0.0, 0.0);
        // c = 2 * damping_ratio * sqrt(m * k)
        let c = 2.0 * self.damping_ratio * ((vertex_a.mass + vertex_b.mass) * self.strength).sqrt();

        let delta = vertex_a.position - vertex_b.position;
        let relative_velocity = vertex_a.velocity - vertex_b.velocity;
        let extention = delta.norm() - self.original_distance;
        // The velocity of the bodies in the direction of each other
        // How fast they are approaching
        let approach_velocity = delta.normalize() * relative_velocity.dot(&delta.normalize());

        // F = -kx - cv
        force +=
            delta.normalize() * extention * -self.strength as f64 - approach_velocity * c as f64;

        // Apply the forces to the couple of bodies
        vertex_a.apply_force(force);
        vertex_b.apply_force(-force);
    }
}
