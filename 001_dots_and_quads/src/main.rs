use nannou::prelude::*;

struct Model {
    passive_dots: Vec<PassiveDot>,
    active_dots: Vec<ActiveDot>,
    passive_dot_count: [usize; 2],
}

pub mod dots;
use dots::*;

fn main() {
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::refresh_sync())
        .run();
}

fn model(app: &App) -> Model {
    let window = app
        .new_window()
        .size(1080, 1080)
        .resizable(false)
        .view(view)
        .build()
        .unwrap();

    let screen_rect = app.window(window).unwrap().rect();

    let (passive_dots, passive_dot_count) = generate_dot_grid(&screen_rect, 30.0);

    Model {
        passive_dots,
        active_dots: generate_active_dots(&screen_rect, 20),
        passive_dot_count,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let delta = update.since_last.as_secs_f32();

    //update_dots(&mut model.passive_dots, delta);
    update_dots(&mut model.active_dots, app, delta);

    model
        .passive_dots
        .iter_mut()
        .for_each(|passive_dot| passive_dot.do_displacement(&model.active_dots));
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);

    let draw = app.draw();

    // Draw the dot grid
    draw_quads(&model.passive_dots, &draw, model.passive_dot_count);
    draw_dots(&model.passive_dots, &draw);

    draw.to_frame(app, &frame).expect("Failed to draw");
}
