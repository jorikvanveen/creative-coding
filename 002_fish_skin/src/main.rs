use nannou::prelude::*;

fn main() {
    println!("Hello, world!");
    nannou::app(model)
        .simple_window(view)
        .loop_mode(LoopMode::RefreshSync)
        .update(update)
        .run();
}

struct Model {}

fn model(app: &App) -> Model {
    Model {}
}

fn update(app: &App, model: &mut Model, update: Update) {

}

fn view(app: &App, model: &Model, frame: Frame) {

}
