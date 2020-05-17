use shipyard::*;
use vermarine_lib::starter::GameState;
use vermarine_lib::input::{ Controls, InputAction, Input };
use InputAction::*;
use Input::*;
use crate::systems::*;
use crate::components::*;

pub fn snake_game() -> GameState {

    //
    // Create World

    let world = World::new();

    world.add_unique(SnakeGame::new());

    world.run(new_game);

    world.add_workload("Snake Game")
        .with_system(system!(move_snake))
        .build();

    world.add_workload("Move Up")
        .with_system(system!(move_up))
        .build();
    
    world.add_workload("Move Down")
        .with_system(system!(move_down))
        .build();

    world.add_workload("Move Left")
        .with_system(system!(move_left))
        .build();

    world.add_workload("Move Right")
        .with_system(system!(move_right))
        .build();

    //
    // Add Controls
    
    let mut controls = Controls::new();

    controls.insert(Pressed(KeyUp), "Move Up");
    controls.insert(Pressed(KeyW), "Move Up");
    
    controls.insert(Pressed(KeyDown), "Move Down");
    controls.insert(Pressed(KeyS), "Move Down");
    
    controls.insert(Pressed(KeyLeft), "Move Left");
    controls.insert(Pressed(KeyA), "Move Left");
    
    controls.insert(Pressed(KeyRight), "Move Right");
    controls.insert(Pressed(KeyD), "Move Right");

    GameState::new("Snake Game", world, controls)
}