pub mod drawing;
pub mod input;
pub mod imgui_piston;
pub mod ui;

use Vector;
use physics::{Vertex, World};

pub enum EditMode {
    Select,
    Create,
}

pub struct ViewState {
    world: World,
    sim_speed: f64,
    physics_dt: f64,

    vertex_scale: f64,
    scale: f64,
    offset: Vector,
    center: Vector,

    edit_mode: EditMode,
    sel_vertex: Option<usize>,
}

impl ViewState {
    pub fn new(world: World) -> ViewState {
        ViewState {
            world,
            sim_speed: 1.0,
            physics_dt: 0.0,
            vertex_scale: 0.25,
            scale: 60.0,
            offset: Vector::new(200.0, 400.0),
            center: Vector::new(0.0, 0.0),
            edit_mode: EditMode::Select,
            sel_vertex: None,
        }
    }

    pub fn to_world_point(&self, point: &Vector) -> Vector {
        Vector::new(
            (point.x + self.offset.x + self.center.x) / self.scale,
            (self.offset.y + self.center.y - point.y) / self.scale,
        )
    }

    fn to_screen_point(&self, point: &Vector) -> Vector {
        Vector::new(
            point.x * self.scale - self.offset.x - self.center.x,
            -point.y * self.scale + self.offset.y + self.center.y,
        )
    }
}
