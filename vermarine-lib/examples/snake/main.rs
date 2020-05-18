mod states;
mod systems;
mod components;

use vermarine_lib::starter::Game;
use crate::states::*;

fn main() {
    Game::new("Snake Example", 800, 800)
        .set_resource_path("vermarine-lib/examples/snake/resources")
        .launch(snake_game());
}