use nannou::color::encoding::Srgb;
use nannou::geom::Quad;
use nannou::geom::Tri;
use nannou::prelude::*;
use rand::prelude::*;

pub trait Dot {
    fn get_position(&self) -> Point2;

    fn get_color(&self) -> rgb::Rgb<Srgb, u8> {
        WHITE
    }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .w(5.0)
            .h(5.0)
            .xy(self.get_position())
            .color(self.get_color());
    }

    fn update(&mut self, _app: &App, _delta: f32) {}
}

#[derive(Debug)]
pub struct ActiveDot {
    pub position: Point2,
    pub velocity: Vec2,
    pub displacement_factor: f32
}

impl Dot for ActiveDot {
    fn get_position(&self) -> Point2 {
        self.position
    }

    fn get_color(&self) -> rgb::Rgb<Srgb, u8> {
        BLACK
    }

    fn update(&mut self, app: &App, delta: f32) {
        let screen = app.window_rect();
        let (next_position, next_velocity) =
            reflect_out_of_bounds(&screen, self.position, self.velocity, delta);
        self.position = next_position;
        self.velocity = next_velocity;
    }
}

#[derive(Debug)]
pub struct PassiveDot {
    pub position: Point2,
    pub original_position: Point2,
}

impl PassiveDot {
    pub fn do_displacement(&mut self, active_dots: &Vec<ActiveDot>) {
        let displacement_vectors: Vec<Vec2> = active_dots
            .iter()
            .map(|active_dot| {
                (self.position - active_dot.position) / (self.position.distance_squared(active_dot.position) + 0.1) * active_dot.displacement_factor
            })
            .collect();

        let final_displacement = displacement_vectors
            .into_iter()
            .reduce(|total, curr| total + curr)
            .unwrap_or(Vec2::new(0.0, 0.0));

        self.position = self.original_position + (final_displacement * 2000.0);
    }
}

impl Dot for PassiveDot {
    fn get_position(&self) -> Point2 {
        self.position
    }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .w(5.0)
            .h(5.0)
            .xy(self.position)
            .color(WHITE)
            .finish();

        //draw.ellipse()
        //    .w(5.0)
        //    .h(5.0)
        //    .xy(self.original_position)
        //    .color(WHITE)
        //    .finish();

        //draw.line()
        //    .start(self.position)
        //    .end(self.original_position)
        //    .color(WHITE)
        //    .stroke_weight(5.0)
        //    .finish();
    }
}

pub fn generate_dot_grid(screen: &Rect, spacing: f32) -> (Vec<PassiveDot>, [usize; 2]) {
    let x_count = (screen.w() / spacing).floor() as usize;
    let y_count = (screen.h() / spacing).floor() as usize;

    let x_margin = (screen.w() - (x_count as f32 - 1.0) * spacing) / 2.0;
    let y_margin = -(screen.h() - (y_count as f32 - 1.0) * spacing) / 2.0;
    let margin = Vec2::new(x_margin, y_margin);

    let start_pos = screen.top_left() + margin;

    let mut dots = vec![];
    for x in 0..x_count {
        for y in 0..y_count {
            let position = start_pos + Vec2::new(x as f32, -(y as f32)) * spacing;

            let dot = PassiveDot {
                position,
                original_position: position,
            };

            dots.push(dot);
        }
    }

    (dots, [x_count, y_count])
}

pub fn generate_active_dots(screen: &Rect, count: usize) -> Vec<ActiveDot> {
    let mut dots = vec![];
    let mut rng = thread_rng();

    for _ in 0..count {
        dots.push(ActiveDot {
            position: Vec2::new(
                rng.gen_range(screen.left()..=screen.right()),
                rng.gen_range(screen.bottom()..=screen.top()),
            ),
            velocity: random::<Vec2>() * 200.0,
            displacement_factor: rng.gen_range(0.9..=1.5)
        })
    }

    dots
}

pub fn draw_dots(dots: &Vec<impl Dot>, draw: &Draw) {
    dots.iter().for_each(|dot| dot.draw(draw));
}

pub fn update_dots(dots: &mut Vec<impl Dot>, app: &App, delta: f32) {
    dots.iter_mut().for_each(|dot| dot.update(app, delta));
}

pub fn draw_quads(dots: &Vec<PassiveDot>, draw: &Draw, dot_count: [usize; 2]) {
    let [dots_x, dots_y] = dot_count;
    let max_idx = dots_x * dots_y - 1;

    for y in 0..(dots_y-1) {
        for x in 0..(dots_x-1) {
            let idx_top_left = y * dots_y + x;
            let idx_top_right = y * dots_y + (x + 1);
            let idx_bottom_left = (y + 1) * dots_y + x;
            let idx_bottom_right = (y + 1) * dots_y + (x + 1);

            if idx_top_left > max_idx
                || idx_top_right > max_idx
                || idx_bottom_left > max_idx
                || idx_bottom_right > max_idx
            {
                continue;
            }

            let top_left = dots[idx_top_left].position;
            let top_right = dots[idx_top_right].position;
            let bottom_left = dots[idx_bottom_left].position;
            let bottom_right = dots[idx_bottom_right].position;

            // Get surface area
            let quad = Quad([top_left, top_right, bottom_right, bottom_left]);
            let tri1 = quad.triangles().0;
            let tri2 = quad.triangles().1;
            let area = triangle_area(tri1) + triangle_area(tri2);
            
            let color = Hsl::new(204.0, 1.0, sigmoid(1.0 / 30.0 * (area.sqrt() - 40.0)));

            draw.quad()
                .points(top_left, top_right, bottom_right, bottom_left)
                .color(color)
                .finish();
        }
    }
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

fn triangle_area(tri: Tri<Vec2>) -> f32 {
    let mut vertices = tri.vertices();
    let v1 = vertices.next().unwrap();
    let v2 = vertices.next().unwrap();
    let v3 = vertices.next().unwrap();

    // a b and c are the three sides of the triangle
    let a = v1.distance(v2);
    let b = v2.distance(v3);
    let c = v3.distance(v1);

    // Use Heron's formula to calculate the area
    let s = (a + b + c) / 2.0;
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

fn sigmoid(x: f32) -> f32 {
    return 1.0 / (1.0 + (-x).exp());
}
