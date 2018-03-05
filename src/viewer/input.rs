use super::*;
use Vector;

use piston_window::*;

pub struct InputState {
    pub cursor: Vector,
    pub last_cursor: Vector,
    pub pressed_mouse: Option<MouseButton>,
    pub held_mouse: Option<MouseButton>,
    pub mouse_wheel: f64,
    pub pressed_keys: Vec<Key>,
    pub held_keys: Vec<Key>,
    pub released_keys: Vec<Key>,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            cursor: Vector::new(0.0, 0.0),
            last_cursor: Vector::new(0.0, 0.0),
            pressed_mouse: None,
            held_mouse: None,
            mouse_wheel: 0.0,
            pressed_keys: Vec::new(),
            held_keys: Vec::new(),
            released_keys: Vec::new(),
        }
    }

    /// Called once all the input is been processed
    /// Sets as held all the keys/mouse buttons that were pressed
    pub fn processed(&mut self) {
        // Everything that was now pressed becomes held
        if self.pressed_mouse.is_some() {
            self.held_mouse = self.pressed_mouse;
            self.pressed_mouse = None;
        }

        // Everything that was now pressed becomes held
        for _ in 0..self.pressed_keys.len() {
            let key = self.pressed_keys.remove(0);
            self.held_keys.push(key);
        }

        // Remove all the released keys
        self.released_keys.clear();

        self.mouse_wheel = 0.0;

        self.last_cursor = self.cursor;
    }

    /// Updates the current Input State
    pub fn event(&mut self, e: &Event) {
        e.mouse_cursor(|x, y| {
            self.cursor.x = x;
            self.cursor.y = y;
        });

        e.mouse_scroll(|_dx, dy| {
            self.mouse_wheel += dy;
        });

        if let Some(Button::Keyboard(key)) = e.press_args() {
            // Add the key only if it wasn't already added
            if !self.pressed_keys.contains(&key) && !self.held_keys.contains(&key) {
                self.pressed_keys.push(key);
            }
        };

        // Remove the release keys form the held and pressed keys
        // (In case one key was pressed and released before the events were processed)
        if let Some(Button::Keyboard(key)) = e.release_args() {
            for i in 0..self.pressed_keys.len() {
                if self.pressed_keys[i] == key {
                    self.pressed_keys.remove(i);
                    break;
                }
            }

            for i in 0..self.held_keys.len() {
                if self.held_keys[i] == key {
                    self.held_keys.remove(i);
                    break;
                }
            }

            self.released_keys.push(key);
        }

        if let Some(Button::Mouse(button)) = e.press_args() {
            self.pressed_mouse = Some(button);
        }

        if let Some(Button::Mouse(_)) = e.release_args() {
            self.pressed_mouse = None;
            self.held_mouse = None;
        }
    }
}

pub fn handle_keyboard(view: &mut ViewState, input: &mut InputState, dt: f64) {
    for key in &input.pressed_keys {
        match *key {
            Key::Space => if view.sim_speed != 0.0 {
                view.sim_speed = 0.0;
            } else {
                view.sim_speed = 1.0;
            },

            Key::Q => view.edit_mode = EditMode::Select,
            Key::C => view.edit_mode = EditMode::Create,
            _ => {}
        }
    }
}

pub fn handle_mouse(view: &mut ViewState, input: &mut InputState) {
    // When the mouse button has just been pressed
    if let Some(button) = input.pressed_mouse {
        let mouse_position = view.to_world_point(&input.cursor);

        if let EditMode::Create = view.edit_mode {
            handle_edit(view, &input, &button);
        }

        // Set the selected vertex to the vertex under the cursor
        view.sel_vertex = view.world
            .get_vertex_at(&mouse_position, view.vertex_scale * 2.0);

        // Set the selected surface to the surface under the cursor
        view.sel_surface = view.world
            .get_surface_at(&mouse_position, view.vertex_scale * 2.0);
    }

    // When the mouse button is being held
    if let Some(button) = input.held_mouse {
        if let EditMode::Select = view.edit_mode {
            handle_select(view, &input, &button);
        }
    }

    handle_move(view, input);

    view.scale += input.mouse_wheel * 5.0;
    if view.scale < 1.0 {
        view.scale = 1.0;
    }
}

fn handle_select(view: &mut ViewState, input: &InputState, button: &MouseButton) {
    let mouse_position = view.to_world_point(&input.cursor);
    if let MouseButton::Left = *button {
        if let Some(index) = view.sel_vertex {
            if view.sim_speed != 0.0 {
                // Move the selected vertex TOWARDS the cursor
                let mut vertex = view.world.verts[index].borrow_mut();
                let position = vertex.position;

                let mut force = mouse_position - position;
                force = force.normalize() * view.pull_force as f64;
                vertex.apply_force(force);
            } else {
                // Move the selected vertex as much as the cursor has moved
                let surfaces = view.world.get_vertex_surfaces(index);
                let mut vertex = view.world.verts[index].borrow_mut();

                let last_mouse = view.to_world_point(&input.last_cursor);
                vertex.position += mouse_position - last_mouse;

                // Adjust the surface distances accordingly
                for i in surfaces {
                    let surface = &mut view.world.surfaces[i];

                    let other_vertex = if surface.index_a == index {
                        view.world.verts[surface.index_b].borrow()
                    } else {
                        view.world.verts[surface.index_a].borrow()
                    };

                    surface.target_distance = (vertex.position - other_vertex.position).norm();
                }
            }
        }
    }
}

fn handle_edit(view: &mut ViewState, input: &InputState, button: &MouseButton) {
    let mouse_position = view.to_world_point(&input.cursor);
    match *button {
        MouseButton::Left => {
            let clicked_vertex = view.world
                .get_vertex_at(&mouse_position, view.vertex_scale * 2.0);
            // If the user clicked on vertex make a surface
            if let Some(index) = clicked_vertex {
                // If there was an vertex already selected make a surface
                if let Some(sel_index) = view.sel_vertex {
                    if sel_index != index {
                        view.world.create_surface(index, sel_index);
                    }
                }
            }
            // If the user clicked on nothing create a new vertex
            else {
                view.world.add_vertex(Vertex::new(mouse_position));
            }
        }
        MouseButton::Right => {
            let clicked_vertex = view.world
                .get_vertex_at(&mouse_position, view.vertex_scale * 2.0);

            // Remove the clicked vertex
            if let Some(vertex_index) = clicked_vertex {
                view.world.remove_vertex(vertex_index);
                view.sel_vertex = None;
            } else {
                // Remove the clicked surface if any
                let clicked_surface = view.world.get_surface_at(&mouse_position, 0.5);
                if let Some(surface_index) = clicked_surface {
                    view.world.surfaces.remove(surface_index);
                    view.sel_surface = None;
                }
            }
        }
        _ => {}
    }
}

fn handle_move(view: &mut ViewState, input: &InputState) {
    if let Some(MouseButton::Right) = input.held_mouse {
        let mut delta = input.last_cursor - input.cursor;
        delta.y = -delta.y;
        view.offset += delta / view.scale;
    }
}
