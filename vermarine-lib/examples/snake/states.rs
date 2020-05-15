use shipyard::*;
use vermarine_lib::starter::GameState;
use vermarine_lib::input::{ Controls, InputAction, Input };
use InputAction::*;
use Input::*;
use crate::systems::*;

pub fn snake_game() -> GameState {
    let world = World::new();
    let controls = Controls::new();

    world.run(new_game);

    world.add_workload("Test").build();

    GameState::new("Test", world, controls)
}