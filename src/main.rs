use std::time::Duration;
use rand::prelude::random;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::{PrimaryWindow, WindowResolution};

/// Colors for the sprites
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
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

/// Size of the sprites
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

/// Event triggerred every time the snake grows
struct GrowthEvent;

/// Event triggerred when the snake touches a wall or itself
struct GameOverEvent;

/// Head of the snake
#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

/// Part of the tail
#[derive(Component)]
struct SnakeSegment;

/// Vector containing the parts of the tail
#[derive(Default, Deref, DerefMut, Resource)]
struct SnakeSegments(Vec<Entity>);

#[derive(Default, Resource)]
struct LastTailPosition(Option<Position>);

pub struct SetupPlugin;

// This plugin handles the logic for greeting users
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        // Startup systems are only called once before other systems
        app.add_startup_system(init_window)
            .add_startup_system(init_camera)
            .add_startup_system(spawn_snake)
            // Makes the food spawn every second
            .add_system(food_spawner.run_if(on_timer(Duration::from_secs_f32(FOOD_SPAWN_TIMESTEP))))
            .add_system(snake_movement.run_if(on_timer(Duration::from_secs_f32(SNAKE_MOVEMENT_TIMESTEP))))
            .add_system(snake_movement_input.before(snake_movement))
            // Inserts the vector of tail segments
            .insert_resource(SnakeSegments::default())
            // The last tail position
            .insert_resource(LastTailPosition::default())
            // Event that is triggered every time the snake is on a Food entity
            .add_event::<GrowthEvent>()
            .add_event::<GameOverEvent>()
            .add_system(game_over.after(snake_movement))
            // Checks if the snake has eaten a Food entity after it has moved
            .add_system(snake_eating.after(snake_movement))
            // Make the snake grow if it ate a Food entity
            .add_system(snake_growth.after(snake_eating))
            // Translates the positions to a grid system
            .add_system(position_translation.in_base_set(CoreSet::PostUpdate))
            // Scales the grid correctly
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
fn spawn_snake(
    mut commands: Commands,
    mut segments: ResMut<SnakeSegments>
) {
    *segments = SnakeSegments(vec![
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..default()
                },
                ..default()
            })
            // Adds the SnakeHead component to the previously spawned entity
            .insert(SnakeHead {
                direction: Direction::Up
            })
            // The head is a segment
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        // Creates the tail
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
}

fn generate_food_position(
    segments_positions_query: Query<&Position, With<SnakeSegment>>
) -> Position {
    let segments_positions = segments_positions_query.iter()
        .collect::<Vec<&Position>>();
    loop {
        let x = (random::<f32>() * ARENA_WIDTH as f32) as i32;
        let y = (random::<f32>() * ARENA_HEIGHT as f32) as i32;

        let res = Position { x, y };
        if !segments_positions.contains(&&res) {
            return res;
        }
    }
}

/// Spawns the food at a random position
fn food_spawner(
    mut commands: Commands,
    query: Query<&Position, With<SnakeSegment>>
) {
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
        .insert(generate_food_position(query))
        // Scales the sprite correctly
        .insert(Size::square(0.8));
}

/// This function computes the scale of the sprites depending on the screen dimension
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
    segments: ResMut<SnakeSegments>,
    mut positions: Query<&mut Position>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        // Collects the segments positions into a vector
        let segment_positions = segments
            .iter()
            // This can be done as *e is the id of the entity
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        // Gets the Position of the head
        let mut head_pos = positions.get_mut(head_entity).unwrap();

        // We change the direction of the snake according to the input
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

        // The player loses if it hits the borders of the map
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.send(GameOverEvent);
        }

        // The player loses if the snake hits itself
        if segment_positions.contains(&head_pos) {
            game_over_writer.send(GameOverEvent);
        }

        // Updates the Position for each SnakeSegment to the previous one
        segment_positions
            .iter()
            // Merges the Positions of the segments and their Entities
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });

        // We assign the resource to the position of the last segment
        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()))
    }
}

/// Handles the GameOverEvent
fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>
) {
    // If the event exists
    if reader.iter().next().is_some() {
        // Deletes every segment of the snake and every food item
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }
        // Re-launches the game
        spawn_snake(commands, segments_res);
    }
    // TODO: Maybe pop-up the main menu ?
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>
) {
    // If there is a triggered event (ie. the snake ate a Food entity)
    if growth_reader.iter().next().is_some() {
        // We spawn a new segment and add it in the list of segments at the last position
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

/// Spawns the segment and returns it
fn spawn_segment(
    mut commands: Commands,
    position: Position
) -> Entity {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: SNAKE_SEGMENT_COLOR,
            ..default()
        },
        ..default()
    })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

/// We check for every food entity on the map if the head is on its position.
/// If it is, the entity despawns and the snake grows.
fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>
) {
    for head_pos in head_positions.iter() {
        for (entity, food_pos) in food_positions.iter() {
            // The snake head is on the same tile as a Food entity
            if food_pos == head_pos {
                // Removes the Food entity
                commands.entity(entity).despawn();
                // Triggers the event that the snake has to grow
                growth_writer.send(GrowthEvent);
            }
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
