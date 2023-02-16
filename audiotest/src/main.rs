use nannou::prelude::*;
use ringbuffer::{RingBuffer, AllocRingBuffer, RingBufferExt};
use nannou_egui::{egui, Egui};
use rayon::prelude::*;

mod wave;

mod audio_spectrum;
use audio_spectrum::Capturer;

mod dot;
use dot::{generate_dots, Dot};

pub struct Config {
    dot_count: usize,
    min_radius: f32,
    max_radius: f32,
    min_period: f32,
    max_period: f32,
    min_amplitude: f32,
    max_amplitude: f32,
    color_factor: f32,
    screen_clearing: f32,
    border_width: f32,
    dot_mode: bool
}

struct AudioData {
    volume_history: Vec<f32>
    //avg: f32,
    //count: usize
}

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    dots: Vec<Dot>,
    config: Config,
    spectrum_recorder: Capturer,
    gui: Egui,
    audio_data: AudioData,
    show_config: bool,
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .view(view)
        .event(event)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let screen = app.window_rect();

    let config = Config {
        dot_count: 500,
        min_radius: 5.0,
        max_radius: 3.0,
        min_period: 0.5,
        max_period: 3.0,
        min_amplitude: 3.0,
        max_amplitude: 10.0,
        color_factor: 2.5,
        screen_clearing: 0.1,
        border_width: 5.0,
        dot_mode: false
    };

    let gui = Egui::from_window(&app.main_window());

    Model {
        dots: generate_dots(&config, &screen),
        spectrum_recorder: Capturer::new("Cool wavy dots".into(), 4096),
        config,
        gui,
        show_config: false,
        audio_data: AudioData {
            volume_history: vec![]
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let mut draw = app.draw();

    if model.config.dot_mode {
        draw = draw.point_mode();
    }

    draw.rect()
        .w(5000.0)
        .h(5000.0)
        .color(rgba(1.0, 1.0, 1.0, model.config.screen_clearing));

    model.dots.iter().for_each(|dot| dot.draw(&draw));
    draw.to_frame(&app, &frame).expect("Failed to draw");

    if !model.show_config {
        return;
    }
    model.gui.draw_to_frame(&frame).expect("Failed to draw gui");
}

fn update(app: &App, model: &mut Model, update: Update) {
    let average_raw_volume = model.audio_data.volume_history
        .clone()
        .into_iter()
        .reduce(|prev, curr| {
            prev + curr
        }).unwrap_or(1.0) / model.audio_data.volume_history.len() as f32;
    // Draw content (circles)
    let time = app.duration.since_start.as_secs_f32();
    let delta = update.since_last.as_secs_f32();
    let spectrum = model.spectrum_recorder.get_spectrum();
    let raw_volume: f32 = spectrum.average().val() + 1e-8; // 0.001 prevents divide by 0
    let volume: f32 = spectrum.average().val() * model.config.color_factor * (1.0/average_raw_volume);
    dbg!(volume);
    let screen = app.window_rect();
    model
        .dots
        .par_iter_mut()
        .for_each(|dot| dot.update(&screen, time, volume, delta));

    if model.audio_data.volume_history.len() > 60*5 {
        model.audio_data.volume_history.remove(0);
    }

    model.audio_data.volume_history.push(raw_volume);

    // Get average raw volume;
    
    dbg!(average_raw_volume);

    // Draw gui
    let gui = &mut model.gui;
    let ctx = gui.begin_frame();

    let window = egui::Window::new("Settings");
    window.show(&ctx, |ui| {
        use egui::{Slider, Checkbox};

        ui.label("Dot count");
        let dot_count = ui
            .add(Slider::new(&mut model.config.dot_count, 1..=2000))
            .changed();

        ui.label("Min amplitude");
        let min_amplitude = ui
            .add(
                Slider::new(
                    &mut model.config.min_amplitude,
                    0.0..=model.config.max_amplitude,
                )
                .clamp_to_range(true),
            )
            .changed();

        ui.label("Max amplitude");
        let max_amplitude = ui
            .add(
                Slider::new(
                    &mut model.config.max_amplitude,
                    model.config.min_amplitude..=200.0,
                )
                .clamp_to_range(true),
            )
            .changed();

        ui.label("Min period");
        let min_period = ui
            .add(
                Slider::new(&mut model.config.min_period, 0.0..=model.config.max_period)
                    .clamp_to_range(true),
            )
            .changed();

        ui.label("Max period");
        let max_period = ui
            .add(
                Slider::new(&mut model.config.max_period, model.config.min_period..=30.0)
                    .clamp_to_range(true),
            )
            .changed();

        ui.label("Screen clearing");
        let _screen_clearing = ui
            .add(Slider::new(&mut model.config.screen_clearing, 0.0..=1.0).clamp_to_range(true))
            .changed();

        ui.label("Border width");
        let _border_width = ui
            .add(Slider::new(&mut model.config.border_width, 0.0..=200.0))
            .changed();

        if dot_count || min_amplitude || max_amplitude || min_period || max_period {
            model.dots = generate_dots(&model.config, &app.window_rect());
        }

        ui.label("Volume factor");
        ui.add(Slider::new(&mut model.config.color_factor, 0.0..=8.0).clamp_to_range(true));

        ui.label("Dot mode");
        ui.add(Checkbox::new(&mut model.config.dot_mode, "Dot mode"));
    });
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        KeyPressed(key) => {
            if key == Key::M || key == Key::S || key == Key::Space {
                model.show_config = !model.show_config;
            }
        }
        _ => {}
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.gui.handle_raw_event(event);
}
