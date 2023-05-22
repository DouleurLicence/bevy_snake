use std::time::Duration;
use rand::prelude::random;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::{PrimaryWindow, WindowResolution};

/// Colors for the sprites
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);

/// Parameters for the game
const FOOD_SPAWN_TIMESTEP: f32 = 1.0;
const SNAKE_MOVEMENT_TIMESTEP: f32 = 0.150;
const ARENA_WIDTH: u32 = 15;
const ARENA_HEIGHT: u32 = 15;

// Bevy uses ECS modeling (Entities, Components, Systems)
#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct Position {
    x: i32,
    y: i32
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Right,
    Up,
    Down
}

/// Returns the opposite direction
impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component)]
struct Size{
    width: f32,
    height: f32
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct Food;

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

pub struct SetupPlugin;

pub struct WindowPlugin;


// This plugin handles the logic for greeting users
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        let mut scheduler = Schedule::new();
        scheduler.add_system(food_spawner);

        // Startup systems are only called once before other systems
        app.add_startup_system(init_window)
            .add_startup_system(init_camera)
            .add_startup_system(spawn_snake)
            // Makes the food spawn every second
            .add_system(food_spawner.run_if(on_timer(Duration::from_secs_f32(FOOD_SPAWN_TIMESTEP))))
            .add_system(snake_movement.run_if(on_timer(Duration::from_secs_f32(SNAKE_MOVEMENT_TIMESTEP))))
            .add_system(snake_movement_input.before(snake_movement))
            .add_system(get_snake_pos)
            .add_system(position_translation.in_base_set(CoreSet::PostUpdate))
            .add_system(size_scaling.in_base_set(CoreSet::PostUpdate));
    }
}

/// Inits the window to have a squared size and a funny title !
fn init_window(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.title = "Rusty Snake :)".to_string();
        window.resolution = WindowResolution::new(
            700.0,
            700.0
        );
    }
}

/// Setup the camera to follow the snake
fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

/// This function spawns the entity with the default parameters of a SpriteBundle
/// except that we change the color and the size of the sprite and transform
fn spawn_snake(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(10.0,10.0,10.0),
                ..default()
            },
            ..default()
        })
        // Adds the SnakeHead component to the previously spawned entity
        .insert(SnakeHead {
            direction: Direction::Up
        })
        .insert(Position { x: 3, y: 3 })
        .insert(Size::square(0.8));
}

/// Spawns the food at a random position
fn food_spawner(mut commands: Commands) {
    // Creates the sprite with the associated color
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: FOOD_COLOR,
            ..default()
        },
        ..default()
    })
        // Insert the Food Component
        .insert(Food)
        // Gives it a Position
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32
        })
        // Scales the sprite correctly
        .insert(Size::square(0.8));
}

/// This function coputes the scale of the sprites depending on the screen dimension
fn size_scaling(
    windows_query: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>
) {
    let Ok(window) = windows_query.get_single() else {return;};
    // We iterate through every sprite and update their scale
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0
        )
    }
}

/// Converts every transform coordinates into their grid coordinates
fn position_translation(
    windows_query: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>
) {

    /// Returns the position in the grid from the float position
    fn convert (pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let Ok(window) = windows_query.get_single() else {return;};
    // Converts every transform item into its grid coordinates
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0
        );
    }
}

// The query here defines on which entities the system will run.
// Here, it will run on every Position component that also have a Size component.
fn get_snake_pos(time: Res<Time>, query: Query<&Position, With<Size>>) {
    for snake in &query {
        println!("Snake is at position: x:{}, y:{}!", snake.x, snake.y)
    }
}

/// Updates the direction of the snake according to the inputs
fn snake_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut heads: Query<&mut SnakeHead>
) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else {
            // Keeps the current direction
            head.direction
        };
        // The snake can't move on its own body
        if dir != head.direction.opposite() {
            head.direction = dir
        }
    }
}

/// Handles the movement of the snake
fn snake_movement(
    mut heads: Query<(&mut Position, &SnakeHead)>
) {
    if let Some((mut head_pos, head)) = heads.iter_mut().next() {
        match head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };
    }
}

// TODO: Need to know how to update the UI and draw food and snake
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SetupPlugin)
        .run();
}
