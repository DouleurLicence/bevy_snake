use bevy::prelude::*;
use bevy::prelude::Projection::Orthographic;

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);

// Bevy uses ECS modeling (Entities, Components, Systems)
#[derive(Component)]
struct Position {
    x: usize,
    y: usize
}

#[derive(Component)]
struct Size(usize);

enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN
}

#[derive(Component)]
struct CurrentDirection(Direction);

struct Food(Position);

#[derive(Bundle)]
struct Snake {
    position: Position,
    size: Size,
    direction: CurrentDirection,
}

#[derive(Component)]
struct SnakeHead;

pub struct SetupPlugin;

// This plugin handles the logic for greeting users
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        // Startup systems are only called once before other systems
        app.add_startup_system(init_camera)
            .add_startup_system(spawn_snake)
            .add_system(snake_movement)
            // TODO: Need to do this every second
            .add_system(get_snake_pos);
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
        .insert(SnakeHead);
}

// The query here defines on which entities the system will run.
// Here, it will run on every Position component that also have a Size component.
fn get_snake_pos(time: Res<Time>, query: Query<&Position, With<Size>>) {
    for snake in &query {
        println!("Snake is at position: x:{}, y:{}!", snake.x, snake.y)
    }
}

/// This function handles the movement of the snake
/// The possible inputs are LEFT, RIGHT, UP and DOWN
fn snake_movement(
    keys: Res<Input<KeyCode>>,
    // I want entities that have SnakeHead, but only their Transform component and not the SnakeHead one
    mut head_positions: Query<&mut Transform, With<SnakeHead>>
) {
    // We need to query the Transform as mut as it will be changed in this system
    for mut transform in head_positions.iter_mut() {
        if keys.pressed(KeyCode::Left) {
            println!("Left is held");
            transform.translation.x -= 2.;
        }
        if keys.pressed(KeyCode::Right) {
            println!("Right is held");
            transform.translation.x += 2.;
        }
        if keys.pressed(KeyCode::Up) {
            println!("Up is held");
            transform.translation.y += 2.;
        }
        if keys.pressed(KeyCode::Down) {
            println!("Down is held");
            transform.translation.y -= 2.;
        }
    }
}

// TODO: Need to know how to update the UI and draw food and snake
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SetupPlugin)
        .run();
}
