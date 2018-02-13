use std::cell::RefCell;
use Vector;

use physics::collisions;
use physics::surface::Surface;

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

pub struct World {
    pub verts: Vec<RefCell<Vertex>>,
    pub surfaces: Vec<Surface>,
    pub debug: DebugView,
}

impl World {
    pub fn new() -> World {
        World {
            verts: Vec::new(),
            surfaces: Vec::new(),
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

        // If there is a surface with the vertex remove it
        let mut i = 0;
        while i < self.surfaces.len() {
            if self.surfaces[i].index_a == index || self.surfaces[i].index_b == index {
                self.surfaces.remove(i);
            } else {
                // Shift all surfaces indexes after the removed element
                if self.surfaces[i].index_a > index {
                    self.surfaces[i].index_a -= 1;
                }
                if self.surfaces[i].index_b > index {
                    self.surfaces[i].index_b -= 1;
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

    pub fn get_vertex_surfaces(&mut self, index: usize) -> Vec<usize> {
        let mut surfaces = Vec::new();
        for surface_i in 0..self.surfaces.len() {
            let surface = &self.surfaces[surface_i];
            if surface.index_a == index || surface.index_b == index {
                surfaces.push(surface_i);
            }
        }
        surfaces
    }

    pub fn get_surface_at(&mut self, position: &Vector, radius: f64) -> Option<usize> {
        for index in 0..self.surfaces.len() {
            let surface = &mut self.surfaces[index];

            let a = self.verts[surface.index_a].borrow_mut();
            let b = self.verts[surface.index_b].borrow_mut();

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

    pub fn create_surface(&mut self, index_a: usize, index_b: usize) {
        use std::usize;

        if index_a == index_b {
            return;
        }

        // Make the first index always the smaller one
        let ord_a = usize::min(index_a, index_b);
        let ord_b = usize::max(index_a, index_b);


        // If the surface is not already present
        for surface in &self.surfaces {
            if surface.index_a == ord_a && surface.index_b == ord_b {
                return;
            }
        }

        // Add the surface to the surfaces
        self.surfaces.push(Surface::new(ord_a, ord_b, &mut self.verts));
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
            for surface in &self.surfaces {
                if surface.index_a != vertex_i && surface.index_b != vertex_i {
                    let mut vertex = self.verts[vertex_i].borrow_mut();
                    let mut segment_a = self.verts[surface.index_a].borrow_mut();
                    let mut segment_b = self.verts[surface.index_b].borrow_mut();

                    if collisions::colliding(&vertex, &segment_a, &segment_b, dt) {
                        collisions::resolve_impulses(&mut vertex, &mut segment_a, &mut segment_b, surface);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f64, iterations: u32, collisions: bool) {
        let dt = dt / iterations as f64;
        for _ in 0..iterations {
            for surface in &self.surfaces {
                surface.apply_force(&mut self.verts);
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

            if collisions {
                for _ in 0..iterations {
                    self.resolve_collisions(dt);
                }
            }

            for i in 0..self.verts.len() {
                let mut vertex = self.verts[i].borrow_mut();
                vertex.update(dt);
            }
        }
    }
}
