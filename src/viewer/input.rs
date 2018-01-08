use super::*;
use Vector;

use piston_window::*;

pub struct InputState {
    pub cursor: Vector,
    pub pressed_mouse: Option<MouseButton>,
    pub held_mouse: Option<MouseButton>,
    pub mouse_wheel: f64,
    pub pressed_keys: Vec<Key>,
    pub held_keys: Vec<Key>,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            cursor: Vector::new(0.0, 0.0),
            pressed_mouse: None,
            held_mouse: None,
            mouse_wheel: 0.0,
            pressed_keys: Vec::new(),
            held_keys: Vec::new(),
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

        self.mouse_wheel = 0.0;
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

    handle_move(view, input, dt);
}

pub fn handle_mouse(view: &mut ViewState, input: &mut InputState, dt: f64) {
    // When the mouse button has just been pressed
    if let Some(button) = input.pressed_mouse {
        let mouse_position = view.to_world_point(&input.cursor);

        if let EditMode::Create = view.edit_mode {
            handle_edit(view, &button, &mouse_position);
        }

        // Set the selected vertex to the vertex under the cursor
        view.sel_vertex = view.world
            .get_vertex_at(&mouse_position, view.vertex_scale * 2.0);
    }

    // When the mouse button is being held
    if let Some(button) = input.held_mouse {
        if let EditMode::Select = view.edit_mode {
            let mouse_position = view.to_world_point(&input.cursor);
            handle_select(view, &button, &mouse_position);
        }
    }

    view.scale += input.mouse_wheel * dt * 100.0;
}

fn handle_select(view: &mut ViewState, button: &MouseButton, mouse_position: &Vector) {
    if let MouseButton::Left = *button {
        if let Some(index) = view.sel_vertex {
            if view.sim_speed != 0.0 {
                // Move the selected vertex TOWARDS the cursor
                let mut vertex = view.world.verts[index].borrow_mut();
                let position = vertex.position;

                let mut force = mouse_position - position;
                force = force.normalize() * 250.0;
                vertex.apply_force(force);
            } else {
                // Move the selected vertex ON the cursor
                let joints = view.world.get_vertex_joints(index);
                let mut vertex = view.world.verts[index].borrow_mut();
                vertex.position = *mouse_position;

                // Adjust the joint distances accordingly
                for i in joints {
                    let joint = &mut view.world.joints[i];

                    let other_vertex = if joint.index_a == index {
                        view.world.verts[joint.index_b].borrow()
                    } else {
                        view.world.verts[joint.index_a].borrow()
                    };

                    joint.original_distance = (vertex.position - other_vertex.position).norm();
                }
            }
        }
    }
}

fn handle_edit(view: &mut ViewState, button: &MouseButton, mouse_position: &Vector) {
    match *button {
        MouseButton::Left => {
            let clicked_vertex = view.world
                .get_vertex_at(mouse_position, view.vertex_scale * 2.0);
            // If the user clicked on vertex make a joint
            if let Some(index) = clicked_vertex {
                // If there was an vertex already selected make a joint
                if let Some(sel_index) = view.sel_vertex {
                    if sel_index != index {
                        view.world.create_joint(index, sel_index);
                    }
                }
            }
            // If the user clicked on nothing create a new vertex
            else {
                view.world.add_vertex(Vertex::new(*mouse_position));
            }
        }
        MouseButton::Right => {
            let clicked_vertex = view.world
                .get_vertex_at(mouse_position, view.vertex_scale * 2.0);

            // Remove the clicked vertex
            if let Some(vertex_index) = clicked_vertex {
                view.world.remove_vertex(vertex_index);
                view.sel_vertex = None;
            } else {
                // Remove the clicked joint if any
                let clicked_joint = view.world.get_joint_at(mouse_position, 0.5);
                if let Some(joint_index) = clicked_joint {
                    view.world.joints.remove(joint_index);
                }
            }
        }
        _ => {}
    }
}

fn handle_move(view: &mut ViewState, input: &InputState, dt: f64) {
    for key in &input.held_keys {
        match *key {
            Key::W => view.offset.y += 200.0 * dt,
            Key::S => view.offset.y -= 200.0 * dt,
            Key::A => view.offset.x -= 200.0 * dt,
            Key::D => view.offset.x += 200.0 * dt,
            _ => {}
        }
    }
}
