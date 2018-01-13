use std::cell::RefCell;
use Vector;

use physics::collisions;

pub struct DebugView {
    pub vectors: Vec<(Vector, Vector)>,
}

pub struct Vertex {
    pub mass: f32,
    pub position: Vector,
    pub velocity: Vector,
    pub acceleration: Vector,
    pub is_static: bool,
}

impl Vertex {
    pub fn new(position: Vector) -> Vertex {
        Vertex {
            mass: 0.05,
            position,
            velocity: Vector::new(0.0, 0.0),
            acceleration: Vector::new(0.0, 0.0),
            is_static: false,
        }
    }

    pub fn apply_force(&mut self, force: Vector) {
        self.acceleration += force / self.mass as f64;
    }

    pub fn force_to_velocity(&mut self, dt: f64) {
        self.velocity += self.acceleration * dt;
    }

    pub fn next_position(&self, dt: f64) -> Vector {
        self.position + self.velocity * dt
    }

    pub fn update(&mut self, dt: f64) {
        if !self.is_static {
            self.position += self.velocity * dt;
        }

        self.acceleration.x = 0.0;
        self.acceleration.y = 0.0;
    }
}

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

pub struct World {
    pub verts: Vec<RefCell<Vertex>>,
    pub joints: Vec<Joint>,
    pub debug: DebugView,
}

impl World {
    pub fn new() -> World {
        World {
            verts: Vec::new(),
            joints: Vec::new(),
            debug: DebugView {
                vectors: Vec::new(),
            },
        }
    }
    

    /*
     #    # ###### #####  ##### ###### #    # ######  ####  
     #    # #      #    #   #   #       #  #  #      #      
     #    # #####  #    #   #   #####    ##   #####   ####  
     #    # #      #####    #   #        ##   #           # 
      #  #  #      #   #    #   #       #  #  #      #    # 
       ##   ###### #    #   #   ###### #    # ######  ####  
    */

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.verts.push(RefCell::new(vertex));
    }

    pub fn remove_vertex(&mut self, index: usize) {
        self.verts.remove(index);

        // If there is a joint with the vertex remove it
        let mut i = 0;
        while i < self.joints.len() {
            if self.joints[i].index_a == index || self.joints[i].index_b == index {
                self.joints.remove(i);
            } else {
                // Shift all joints indexes after the removed element
                if self.joints[i].index_a > index {
                    self.joints[i].index_a -= 1;
                }
                if self.joints[i].index_b > index {
                    self.joints[i].index_b -= 1;
                }

                i += 1;
            }
        }
    }

    pub fn get_vertex_at(&mut self, position: &Vector, radius: f64) -> Option<usize> {
        for index in 0..self.verts.len() {
            let vertex = self.verts[index].borrow();
            if (vertex.position - position).norm() < radius {
                return Some(index);
            }
        }
        None
    }

    /*
          #  ####  # #    # #####  ####  
          # #    # # ##   #   #   #      
          # #    # # # #  #   #    ####  
          # #    # # #  # #   #        # 
     #    # #    # # #   ##   #   #    # 
      ####   ####  # #    #   #    ####  
    */

    pub fn get_vertex_joints(&mut self, index: usize) -> Vec<usize> {
        let mut joints = Vec::new();
        for joint_i in 0..self.joints.len() {
            let joint = &self.joints[joint_i];
            if joint.index_a == index || joint.index_b == index {
                joints.push(joint_i);
            }
        }
        joints
    }

    pub fn get_joint_at(&mut self, position: &Vector, radius: f64) -> Option<usize> {
        for index in 0..self.joints.len() {
            let joint = &mut self.joints[index];

            let a = self.verts[joint.index_a].borrow_mut();
            let b = self.verts[joint.index_b].borrow_mut();

            let delta = position - a.position;
            let segment = a.position - b.position;

            let projection = segment * segment.dot(&delta) / segment.norm().powi(2);

            let collision_point = a.position + projection;
            let distance = (collision_point - position).norm();
            if distance < radius && collision_point.x >= f64::min(a.position.x, b.position.x) && collision_point.x <= f64::max(a.position.x, b.position.x) {
                return Some(index);
            }
        }

        None
    }

    pub fn create_joint(&mut self, index_a: usize, index_b: usize) {
        use std::usize;

        if index_a == index_b {
            return;
        }

        // Make the first index always the smaller one
        let ord_a = usize::min(index_a, index_b);
        let ord_b = usize::max(index_a, index_b);


        // If the joint is not already present
        for joint in &self.joints {
            if joint.index_a == ord_a && joint.index_b == ord_b {
                return;
            }
        }

        // Add the joint to the joints
        self.joints.push(Joint::new(ord_a, ord_b, &mut self.verts));
    }

    /*
      ####  # #    # #    # #        ##   ##### #  ####  #    # 
     #      # ##  ## #    # #       #  #    #   # #    # ##   # 
      ####  # # ## # #    # #      #    #   #   # #    # # #  # 
          # # #    # #    # #      ######   #   # #    # #  # # 
     #    # # #    # #    # #      #    #   #   # #    # #   ## 
      ####  # #    #  ####  ###### #    #   #   #  ####  #    # 
    */

    pub fn resolve_collisions(&mut self, dt: f64) {
        for vertex_i in 0..self.verts.len() {
            for joint_i in 0..self.joints.len() {
                let joint = &mut self.joints[joint_i];
                if joint.index_a != vertex_i && joint.index_b != vertex_i {
                    let mut vertex = self.verts[vertex_i].borrow_mut();
                    let mut segment_a = self.verts[joint.index_a].borrow_mut();
                    let mut segment_b = self.verts[joint.index_b].borrow_mut();

                    if collisions::colliding(&vertex, &segment_a, &segment_b, dt) {
                        collisions::resolve_impulses(&mut vertex, &mut segment_a, &mut segment_b);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f64, iterations: u32) {
        let dt = dt / iterations as f64;
        for _ in 0..iterations {
            for joint in &self.joints {
                joint.apply_force(&mut self.verts);
            }

            for i in 0..self.verts.len() {
                let mut vertex = self.verts[i].borrow_mut();
                if !vertex.is_static {
                    vertex.acceleration.y -= 9.8;
                    vertex.force_to_velocity(dt);
                } else {
                    vertex.velocity.x = 0.0;
                    vertex.velocity.y = 0.0;
                    vertex.acceleration.x = 0.0;
                    vertex.acceleration.y = 0.0;
                }
            }

            for _ in 0..iterations {
                self.resolve_collisions(dt);
            }

            for i in 0..self.verts.len() {
                let mut vertex = self.verts[i].borrow_mut();

                vertex.update(dt);

                if vertex.position.y < 0.0 {
                    vertex.position.y = 0.0;
                    vertex.velocity.y = -vertex.velocity.y;
                    vertex.velocity.x *= 6.0 * dt;
                }
            }
        }
    }
}
