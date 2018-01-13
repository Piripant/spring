use std::cell::RefCell;
use physics::simulation::Vertex;
use Vector;


pub struct Joint {
    pub index_a: usize,
    pub index_b: usize,

    pub damping_ratio: f32,
    pub joint_strength: f32,
    pub original_distance: f64,
}

impl Joint {
    pub fn new(a: usize, b: usize, verts: &mut Vec<RefCell<Vertex>>) -> Joint {
        let vertex_a = verts[a].borrow();
        let vertex_b = verts[b].borrow();

        Joint {
            index_a: a,
            index_b: b,
            damping_ratio: 1.0,
            joint_strength: 40.0,
            original_distance: (vertex_a.position - vertex_b.position).norm(),
        }
    }

    pub fn apply_force(&self, verts: &mut Vec<RefCell<Vertex>>) {
        let mut vertex_a = verts[self.index_a].borrow_mut();
        let mut vertex_b = verts[self.index_b].borrow_mut();

        let mut force = Vector::new(0.0, 0.0);
        // c = 2 * damping_ratio * sqrt(m * k)
        let c = 2.0 * self.damping_ratio
            * ((vertex_a.mass + vertex_b.mass) * self.joint_strength).sqrt();

        let delta = vertex_a.position - vertex_b.position;
        let relative_velocity = vertex_a.velocity - vertex_b.velocity;
        let extention = delta.norm() - self.original_distance;
        // The velocity of the bodies in the direction of each other
        // How fast they are approaching
        let joint_velocity = delta.normalize() * relative_velocity.dot(&delta.normalize());

        // F = -kx - cv
        force +=
            delta.normalize() * extention * -self.joint_strength as f64 - joint_velocity * c as f64;

        // Apply the forces to the couple of bodies
        vertex_a.apply_force(force);
        vertex_b.apply_force(-force);
    }
}
