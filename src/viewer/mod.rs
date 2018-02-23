pub mod drawing;
pub mod input;
pub mod imgui_piston;
pub mod ui;

use Vector;
use physics::simulation::{Vertex, World};

pub enum EditMode {
    Select,
    Create,
}

pub struct ViewState {
    world: World,
    sim_speed: f64,
    physics_dt: f64,
    iterations: u32,
    collisions: bool,
    pull_force: f32,

    vertex_scale: f64,
    scale: f64,
    offset: Vector,
    window_size: Vector,

    edit_mode: EditMode,
    sel_vertex: Option<usize>,
    sel_surface: Option<usize>,
}

impl ViewState {
    pub fn new(world: World) -> ViewState {
        ViewState {
            world,
            sim_speed: 1.0,
            physics_dt: 0.0,
            iterations: 8,
            collisions: true,
            pull_force: 250.0,
            vertex_scale: 0.25,
            scale: 60.0,
            offset: Vector::new(0.0, 0.0),
            window_size: Vector::new(0.0, 0.0),
            edit_mode: EditMode::Select,
            sel_vertex: None,
            sel_surface: None,
        }
    }

    fn to_screen_point(&self, point: &Vector) -> Vector {
        Vector::new(
            (point.x - self.offset.x) * self.scale + self.window_size.x / 2.0,
            -(point.y - self.offset.y) * self.scale + self.window_size.y / 2.0,
        )
    }

    pub fn to_world_point(&self, point: &Vector) -> Vector {
        Vector::new(
            (point.x - self.window_size.x / 2.0) / self.scale + self.offset.x,
            (-point.y + self.window_size.y / 2.0) / self.scale + self.offset.y,
        )
    }
}
