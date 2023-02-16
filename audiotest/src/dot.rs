use crate::wave::SinWave;
use nannou::prelude::*;

pub struct Dot {
    position: Point2,
    base_color: Hsl,
    color: Hsl,
    velocity: Vec2,
    sin_wave: SinWave,
    radius: f32,
    position_offset: Vec2,
}

impl Dot {
    pub fn update(&mut self, screen: &Rect, time: f32, volume: f32, delta: f32) {
        self.radius = self.sin_wave.evaluate(time) + volume;

        self.position += self.velocity * delta;

        (self.position, self.velocity) =
            reflect_out_of_bounds(screen, self.position, self.velocity, delta);

        self.color = hsl(
            ((self.base_color.hue.to_positive_degrees() / 360.0) + volume / 30.0).into(),
            self.base_color.saturation,
            self.base_color.lightness,
        );

        let dist_from_center = self.position.distance(Point2::ZERO);
        let displacement_factor = 1.0 / dist_from_center;
        let displacement_vec: Vec2 = self.position * displacement_factor * volume * 10.0;
        self.position_offset = displacement_vec;
    }

    pub fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .radius(self.radius)
            .xy(self.position + self.position_offset)
            .color(rgba(1.0, 1.0, 1.0, 0.0)) // Transparent
            .stroke(self.color)
            .stroke_weight(5.0)
            .finish();

        //draw.ellipse()
        //    .radius(clamp_min(self.radius - 5.0, 0.0))
        //    .xy(self.position + self.position_offset)
        //    .color(WHITE)
        //    .finish();
    }
}

pub fn generate_dots(config: &crate::Config, screen: &Rect) -> Vec<Dot> {
    let mut dots = vec![];

    for _ in 0..config.dot_count {
        let x = random_range(screen.left(), screen.right());
        let y = random_range(screen.bottom(), screen.top());
        let position = Point2::new(x, y);
        let radius = random_range(config.min_radius, config.max_radius);
        let sin_wave = SinWave::new(
            random_range(config.min_amplitude, config.max_amplitude),
            random_range(config.min_period, config.max_period),
            0.0,
        );

        let velocity = Vec2::new(random_range(-50.0, 50.0), random_range(-50.0, 50.0));

        let base_color = hsl(random_range(180.0, 220.0) / 360.0, 0.5, 0.5);

        dots.push(Dot {
            position,
            radius,
            sin_wave,
            position_offset: Vec2::ZERO,
            velocity,
            base_color,
            color: base_color,
        });
    }

    dots
}

/// Checks if a given position and velocity will be out of bounds after rendering.
/// Returns a new position and velocity, these will be the same as the passed in values
/// if the object is not out of bounds
fn reflect_out_of_bounds(
    screen: &Rect,
    position: Point2,
    velocity: Vec2,
    delta: f32,
) -> (Point2, Vec2) {
    let next_position = position + velocity * delta;

    if screen.contains(next_position) {
        return (next_position, velocity);
    }

    let mut reflected_velocity = velocity;
    let mut reflected_position = position;

    if next_position.x > screen.right() {
        reflected_velocity.x = -reflected_velocity.x;
        reflected_position.x = screen.right();
    }

    if next_position.x < screen.left() {
        reflected_velocity.x = -reflected_velocity.x;
        reflected_position.x = screen.left();
    }

    if next_position.y > screen.top() {
        reflected_velocity.y = -reflected_velocity.y;
        reflected_position.y = screen.top();
    }

    if next_position.y < screen.bottom() {
        reflected_velocity.y = -reflected_velocity.y;
        reflected_position.y = screen.bottom();
    }

    (reflected_position, reflected_velocity)
}
