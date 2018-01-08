use std::time::SystemTime;

use super::*;
use piston_window::*;
use imgui;
use viewer::imgui_piston::{Renderer, Shaders};
use viewer::input::InputState;

pub fn view_loop(mut view: ViewState) {
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Spring", [1280, 720])
        .samples(4)
        .opengl(opengl)
        .build()
        .unwrap();

    let mut gui = imgui::ImGui::init();
    let mut gui_renderer = Renderer::init(
        &mut gui,
        &mut window.factory,
        Shaders::GlSl130,
        window.output_color.clone(),
    ).expect("Failed to initialize GUI renderer");

    ui::configure_keys(&mut gui);

    let mut input_state = InputState::new();

    // The clock used to mesure the simulation timestep
    let mut time = SystemTime::now();
    // The clock used to measure the rendering timestep
    let mut render_time = SystemTime::now();
    // The targeted time difference between rendering frames (inverse of target framerate)
    let target_dt = 1.0 / 90.0;
    while let Some(e) = window.next() {
        // Updating the input state
        input_state.event(&e);

        // Stepping the world simulation
        view.world.debug.vectors.clear();
        view.physics_dt = get_elapsed(&time);
        view.world.update(view.physics_dt * view.sim_speed, 8);
        time = SystemTime::now();

        // When the window is resize the gui renderer must be regeretated
        // With the new window.factory containing the new height and width
        if let Some(_args) = e.resize_args() {
            gui_renderer = Renderer::init(
                &mut gui,
                &mut window.factory,
                Shaders::GlSl130,
                window.output_color.clone(),
            ).expect("Failed to initialize GUI renderer");
        }


        if let Some(args) = e.render_args() {
            let dt = get_elapsed(&render_time);
            if dt >= target_dt {
                // Reset the clock
                render_time = SystemTime::now();
                //view.center.y = window.size().height as f64;
                //view.center.x = window.size().width as f64;

                window.draw_2d(&e, |c, g| {
                    clear([1.0; 4], g);

                    // Drawing the joints
                    for joint in &view.world.joints {
                        let vertex_a = view.world.verts[joint.index_a].borrow();
                        let vertex_b = view.world.verts[joint.index_b].borrow();

                        let position_a = view.to_screen_point(&vertex_a.position);
                        let position_b = view.to_screen_point(&vertex_b.position);

                        let line_data = [position_a.x, position_a.y, position_b.x, position_b.y];

                        line([1.0, 0.0, 0.0, 1.0], 1.0, line_data, c.transform, g);
                    }

                    // Drawing the vertexes
                    for i in 0..view.world.verts.len() {
                        let vertex = view.world.verts[i].borrow();
                        let mut color = [0.0, 0.0, 1.0, 1.0];

                        // If this is the selected vextex set the color to green
                        if let Some(sel_index) = view.sel_vertex {
                            if sel_index == i {
                                color = [0.0, 1.0, 0.0, 1.0];
                            }
                        }

                        let position = view.to_screen_point(&vertex.position);
                        let rect =
                            ellipse::circle(position.x, position.y, view.vertex_scale * view.scale);
                        ellipse(color, rect, c.transform, g);
                    }

                    // Drawing the debug vectors
                    for vector in &view.world.debug.vectors {
                        let start = vector.0;
                        let vec = vector.1;

                        let line_data = [
                            view.to_screen_point(&start).x,
                            view.to_screen_point(&start).y,
                            view.to_screen_point(&(start + vec)).x,
                            view.to_screen_point(&(start + vec)).y,
                        ];

                        line([0.0, 1.0, 1.0, 1.0], 1.0, line_data, c.transform, g);
                    }

                    let y = view.to_screen_point(&Vector::new(0.0, 0.0)).y;
                    let ground_line = [0.0, y, args.draw_width as f64, y];
                    line([0.0, 0.0, 0.0, 1.0], 1.0, ground_line, c.transform, g);
                });

                ui::update_mouse(&mut gui, window.size(), window.draw_size(), &input_state);
                let mut ui = gui.frame(
                    (args.draw_width, args.draw_height),
                    (args.draw_width, args.draw_height),
                    dt as f32,
                );
                // Process the input events only if they are not being processed alredy by the UI
                let (capture_mouse, capture_keyboard) = ui::run_ui(&mut ui, &mut view);
                if !capture_keyboard {
                    input::handle_keyboard(&mut view, &mut input_state, dt);
                }
                if !capture_mouse {
                    input::handle_mouse(&mut view, &mut input_state, dt);
                }
                input_state.processed();

                gui_renderer
                    .render(ui, &mut window.factory, &mut window.encoder)
                    .expect("GUI rendering failed");

                // Do the actual rendering of the UI
                window.encoder.flush(&mut window.device);
            }
        }
    }
}

fn get_elapsed(time: &SystemTime) -> f64 {
    let elapsed = time.elapsed().unwrap();
    elapsed.as_secs() as f64 + f64::from(elapsed.subsec_nanos()) * 1e-9
}
