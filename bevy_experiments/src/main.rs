use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::log::prelude::*;
use rand::prelude::*;

const DOT_COUNT: usize = 100;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Resource)]
struct ScreenRect(Option<Rect>);

fn update_screen_rect(windows: Res<Windows>, mut screen_rect: ResMut<ScreenRect>) {
    let window = windows.primary();
    screen_rect.0 = Some(Rect::from_center_size(Vec2::ZERO, Vec2::new(window.width(), window.height())));
}

fn apply_velocity(
    mut query: Query<(&mut Velocity, &mut Transform, Option<&ScreenReflect>)>,
    screen_rect: Res<ScreenRect>,
    time: Res<Time>
) {
    let delta = time.delta().as_secs_f32();
    let screen_rect = screen_rect.0.unwrap();

    for (mut velocity, mut transform, screen_reflect) in query.iter_mut() {
        let mut new_translation = transform.translation + velocity.0.extend(0.) * delta;
        let mut new_velocity = velocity.0;

        if screen_reflect.is_some() && !screen_rect.contains(new_translation.truncate()) {
            // Do screen reflection
            (new_translation, new_velocity) = match new_translation {
                _ if new_translation.x > screen_rect.max.x => {
                    (Vec3::new(screen_rect.max.x, new_translation.y, 0.0), Vec2::new(-new_velocity.x, new_velocity.y))
                },
                _ if new_translation.x < screen_rect.min.x => {
                    (Vec3::new(screen_rect.min.x, new_translation.y, 0.0), Vec2::new(-new_velocity.x, new_velocity.y))
                },
                _ if new_translation.y > screen_rect.max.y => {
                    (Vec3::new(new_translation.x, screen_rect.max.y, 0.0), Vec2::new(new_velocity.x, -new_velocity.y))
                },
                _ if new_translation.y < screen_rect.min.y => {
                    (Vec3::new(new_translation.x, screen_rect.min.y, 0.0), Vec2::new(new_velocity.x, -new_velocity.y))
                }
                _ => (new_translation, new_velocity)
            }
        }

        velocity.0 = new_velocity;
        transform.translation = new_translation;
    }
}

#[derive(Component)]
struct ScreenReflect;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ScreenRect(None))
        .add_startup_system(update_screen_rect)
        .add_startup_system(setup.after(update_screen_rect))
        .add_system(update_screen_rect)
        .add_system(apply_velocity)
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    screen_rect: Res<ScreenRect>,
) {
    let screen_rect = screen_rect.0.unwrap();
    commands.spawn(Camera2dBundle::default());

    let mut rng = thread_rng();

    let mesh = meshes.add(shape::Circle::new(1.0).into());
    let material = color_materials.add(ColorMaterial {
        color: Color::rgb(1.0, 1.0, 1.0),
        ..ColorMaterial::default()
    });
    
    // Generate dots
    for _ in 0..DOT_COUNT {
        // Generate random position
        let velocity = Velocity(Vec2::new(
            rng.gen_range(-100.0..=100.),
            rng.gen_range(-100.0..=100.)
        ));

        let position = Vec2::new(
            rng.gen_range(screen_rect.min.x..=screen_rect.max.x),
            rng.gen_range(screen_rect.min.y..=screen_rect.max.y)
        );

        let size = rng.gen_range(10.0..40.0);

        let entity = (
            velocity,
            ScreenReflect,
            MaterialMesh2dBundle {
                transform: Transform::from_translation(position.extend(0.)).with_scale(Vec3::ONE * size),
                mesh: mesh.clone().into(),
                material: material.clone(),
                ..MaterialMesh2dBundle::default()
            }
        );

        commands.spawn(entity);
    };
}
