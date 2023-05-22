use bevy::prelude::*;

const TIMESTEP_2_PER_SECOND: f64 = 30.0 / 60.0;

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

pub struct SetupPlugin;

// This plugin handles the logic for greeting users
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        // Startup systems are only called once before other systems
        app.add_startup_system(init_snake)
            .add_system(keyboard_input)
            // TODO: Need to do this every second
            .add_system(get_snake_pos);
    }
}

// Every function is a system in Bevy
fn init_snake(mut commands: Commands) {
    commands.spawn((Snake{
        position: Position{x: 0, y: 0},
        size: Size(2),
        direction: CurrentDirection(Direction::DOWN),
    }));
}

// The query here defines on which entities the system will run.
// Here, it will run on every Position component that also have a Size component.
fn get_snake_pos(time: Res<Time>, query: Query<&Position, With<Size>>) {
    for snake in &query {
        println!("Snake is at position: x:{}, y:{}!", snake.x, snake.y)
    }
}

// TODO: Need to get access to the snake object
fn keyboard_input(
    keys: Res<Input<KeyCode>>,
) {
    if keys.pressed(KeyCode::Left) {
        println!("Left is held")

    }
    if keys.pressed(KeyCode::Right) {
        println!("Right is held")
    }
    if keys.pressed(KeyCode::Up) {
        println!("Up is held")
    }
    if keys.pressed(KeyCode::Down) {
        println!("Down is held")
    }
}

// TODO: Need to know how to update the UI and draw food and snake
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SetupPlugin)
        .run();
}
