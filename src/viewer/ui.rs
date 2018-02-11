use super::*;
use super::input::InputState;

use imgui::*;
use piston_window::*;

pub fn run_ui(ui: &mut Ui, view: &mut ViewState) -> (bool, bool) {
    let mut sim_speed = view.sim_speed as f32;
    let mut vertex_scale = view.vertex_scale as f32;

    ui.window(im_str!("Simulation Settings"))
        .size((300.0, 100.0), ImGuiCond::FirstUseEver)
        .build(|| {
            ui.text(im_str!("Physics framerate: {}", 1.0 / view.physics_dt));
            ui.text(im_str!("Bodies: {}", view.world.verts.len()));
            ui.text(im_str!("Surfaces: {}", view.world.surfaces.len()));

            ui.separator();

            let speed_slider =
                ui.slider_float(im_str!("Simulation speed"), &mut sim_speed, 0.0, 1.0);
            speed_slider.build();

            let vscale_slider = ui.slider_float(
                im_str!("Vertex Graphical Size"),
                &mut vertex_scale,
                0.0,
                1.0,
            );
            vscale_slider.build();
        });

    view.sim_speed = sim_speed as f64;
    view.vertex_scale = vertex_scale as f64;


    if let Some(index) = view.sel_vertex {
        let mut vertex = view.world.verts[index].borrow_mut();
        ui.window(im_str!("Vertex"))
            .size((300.0, 600.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("ID: {}", index));
                ui.text(im_str!(
                    "Position: {:.2}, {:.2}",
                    vertex.position.x,
                    vertex.position.y
                ));
                ui.text(im_str!(
                    "Velocity: {:.2}, {:.2}",
                    vertex.velocity.x,
                    vertex.velocity.y
                ));

                ui.input_float(im_str!("Mass"), &mut vertex.mass).build();
                ui.checkbox(im_str!("Static"), &mut vertex.is_static);
            });
    }

    if let Some(index) = view.sel_surface {
        let surface = &mut view.world.surfaces[index];
        ui.window(im_str!("Surface"))
            .size((300.0, 600.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("ID: {}", index));
                ui.text(im_str!("Vertex A: {}", surface.index_a));
                ui.text(im_str!("Vertex B: {}", surface.index_b));

                ui.text(im_str!("Target distance: {}", surface.target_distance));

                ui.input_float(im_str!("Damping ratio"), &mut surface.damping_ratio)
                    .build();
                ui.input_float(im_str!("Strength"), &mut surface.strength)
                    .build();
                ui.input_float(im_str!("Friction"), &mut surface.friction)
                    .build();
                ui.input_float(im_str!("Resitution"), &mut surface.resitution)
                    .build();
            });
    }

    (ui.want_capture_mouse(), ui.want_capture_keyboard())
}

pub fn configure_keys(imgui: &mut ImGui) {
    use imgui::ImGuiKey;

    imgui.set_imgui_key(ImGuiKey::Tab, 0);
    imgui.set_imgui_key(ImGuiKey::LeftArrow, 1);
    imgui.set_imgui_key(ImGuiKey::RightArrow, 2);
    imgui.set_imgui_key(ImGuiKey::UpArrow, 3);
    imgui.set_imgui_key(ImGuiKey::DownArrow, 4);
    imgui.set_imgui_key(ImGuiKey::PageUp, 5);
    imgui.set_imgui_key(ImGuiKey::PageDown, 6);
    imgui.set_imgui_key(ImGuiKey::Home, 7);
    imgui.set_imgui_key(ImGuiKey::End, 8);
    imgui.set_imgui_key(ImGuiKey::Delete, 9);
    imgui.set_imgui_key(ImGuiKey::Backspace, 10);
    imgui.set_imgui_key(ImGuiKey::Enter, 11);
    imgui.set_imgui_key(ImGuiKey::Escape, 12);
    imgui.set_imgui_key(ImGuiKey::A, 13);
    imgui.set_imgui_key(ImGuiKey::C, 14);
    imgui.set_imgui_key(ImGuiKey::V, 15);
    imgui.set_imgui_key(ImGuiKey::X, 16);
    imgui.set_imgui_key(ImGuiKey::Y, 17);
    imgui.set_imgui_key(ImGuiKey::Z, 18);
}

pub fn update_keyboard(imgui: &mut ImGui, input: &InputState) {
    set_keys(imgui, &input.pressed_keys, true);
    set_keys(imgui, &input.released_keys, false);
}

fn set_keys(imgui: &mut ImGui, keys: &Vec<Key>, state: bool) {
    for key in keys {
        match *key {
            Key::Tab => imgui.set_key(0, state),
            Key::Left => imgui.set_key(1, state),
            Key::Right => imgui.set_key(2, state),
            Key::Up => imgui.set_key(3, state),
            Key::Down => imgui.set_key(4, state),
            Key::PageUp => imgui.set_key(5, state),
            Key::PageDown => imgui.set_key(6, state),
            Key::Home => imgui.set_key(7, state),
            Key::End => imgui.set_key(8, state),
            Key::Delete => imgui.set_key(9, state),
            Key::Backspace => imgui.set_key(10, state),
            Key::Return => imgui.set_key(11, state),
            Key::Escape => imgui.set_key(12, state),
            Key::A => imgui.set_key(13, state),
            Key::C => imgui.set_key(14, state),
            Key::V => imgui.set_key(15, state),
            Key::X => imgui.set_key(16, state),
            Key::Y => imgui.set_key(17, state),
            Key::Z => imgui.set_key(18, state),
            Key::LCtrl | Key::RCtrl => imgui.set_key_ctrl(state),
            Key::LShift | Key::RShift => imgui.set_key_shift(state),
            Key::LAlt | Key::RAlt => imgui.set_key_alt(state),
            //Key::LWin | Key::RWin => imgui.set_key_super(state),
            _ => {}
        }

        if key.code() as u8 <= 127 && state {
            let code = key.code() as u8;
            imgui.add_input_character(code as char);
        }
    }
}

pub fn update_mouse(imgui: &mut ImGui, window_size: Size, draw_size: Size, input: &InputState) {
    let scale = imgui.display_framebuffer_scale();
    let scale_x = draw_size.width as f64 / window_size.width as f64;
    let scale_y = draw_size.height as f64 / window_size.height as f64;

    let mouse_x = input.cursor.x * scale_x;
    let mouse_y = input.cursor.y * scale_y;
    imgui.set_mouse_pos(mouse_x as f32 / scale.0, mouse_y as f32 / scale.1);

    let mut mouse_0 = false;
    let mut mouse_1 = false;
    let mut mouse_2 = false;

    match input.pressed_mouse {
        Some(MouseButton::Left) => mouse_0 = true,
        Some(MouseButton::Right) => mouse_1 = true,
        Some(MouseButton::Middle) => mouse_2 = true,
        _ => {}
    }

    match input.held_mouse {
        Some(MouseButton::Left) => mouse_0 = true,
        Some(MouseButton::Right) => mouse_1 = true,
        Some(MouseButton::Middle) => mouse_2 = true,
        _ => {}
    }

    imgui.set_mouse_down(&[mouse_0, mouse_1, mouse_2, false, false]);

    imgui.set_mouse_wheel(input.mouse_wheel as f32);
}
