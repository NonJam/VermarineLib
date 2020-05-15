use std::collections::HashMap;
use std::path::Path;
use shipyard::*;
use tetra::graphics::DrawParams;
use tetra::graphics::{self, Color, Texture};
use tetra::math::Vec2;
use tetra::{ input, Context, ContextBuilder, Result, State, Trans };
use tetra::input::*;
use crate::input::*;
use crate::components::*;
use InputAction::*;
use Input::*;

mod resources;
use resources::*;

pub mod components;
use components::*;

//
// Game

pub struct Game {
    pub title: String,
    pub window_width: i32,
    pub window_height: i32,
    resource_directory: &'static str
}

impl Game {
    pub fn new<S>(title: S, window_width: i32, window_height: i32) -> Self 
        where S: std::string::ToString {
        Game { 
            title: title.to_string(), 
            window_width, 
            window_height, 
            resource_directory: "resources"
        }
    }

    pub fn set_resource_path(&mut self, path: &'static str) -> &mut Self {
        self.resource_directory = path;
        self
    }

    pub fn launch(&self, state: GameState) {
        if let Err(err) = ContextBuilder::new(&self.title, self.window_width, self.window_height)
            .show_mouse(true)
            .build()
            .unwrap()
            .run(|_| Ok(state), |ctx| Ok(Resources::load(ctx,self.resource_directory))) 
            {
                panic!(err)
            }
    }
}

//
// GameState

pub struct GameState {
    workload: String,
    world: World,
    controls: Controls
}

impl State<Resources> for GameState {
    fn update(&mut self, ctx: &mut Context, _: &mut Resources) -> Result<Trans<Resources>> {
        self.handle_input(ctx);
        self.world.run_workload(&self.workload);

        // Right now there's no transitions since there's no way to access this outside the game state
        // I want to add a transition handler that we can pull out of the shipyard world
        Ok(Trans::None)
    }

    fn draw(&mut self, ctx: &mut Context, resources: &mut Resources) -> tetra::Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.world.run(|transforms: View<Transform>, sprites: View<Sprite>|{

            for (&transform, &sprite) in (&transforms, &sprites).iter() {

                let texture = resources.textures.get(&sprite.texture.to_string()).unwrap();

                let center_x = texture.width() as f32 / 2f32;
                let center_y = texture.height() as f32 / 2f32;

                let params = DrawParams::new()
                    .position(Vec2::new(transform.x as f32, transform.y as f32))
                    .origin(Vec2::new(center_x, center_y));

                graphics::draw(ctx, texture, params);

            }
        });

        Ok(())
    }
}

impl GameState {
    pub fn new<S: std::string::ToString>(workload: S, world: World, controls: Controls) -> GameState {
        GameState { workload: workload.to_string(), world, controls }
    }

    // Yikes
    // Later on I wanna transform this into an input Context and throw it into the world for system access
    // For now though this works to map workloads onto key actions
    fn handle_input(&mut self, ctx: &Context) {
        for key in input::get_keys_pressed(ctx) {
            if self.controls.contains_key(&Pressed(Input::from_tetra_key(*key))) {
                self.world.run_workload(&self.controls[&Pressed(Input::from_tetra_key(*key))]);
            }
        }
        for key in input::get_keys_down(ctx) {
            if self.controls.contains_key(&Held(Input::from_tetra_key(*key))) {
                self.world.run_workload(&self.controls[&Held(Input::from_tetra_key(*key))]);
            }
        }
        for key in input::get_keys_released(ctx) {
            if self.controls.contains_key(&Released(Input::from_tetra_key(*key))) {
                self.world.run_workload(&self.controls[&Released(Input::from_tetra_key(*key))]);
            }
        }
        if input::is_mouse_button_pressed(ctx, MouseButton::Left) {
            if self.controls.contains_key(&Pressed(MouseLeft)) {
                self.world.run_workload(&self.controls[&Pressed(MouseLeft)]);
            }
        }
        if input::is_mouse_button_pressed(ctx, MouseButton::Middle) {
            if self.controls.contains_key(&Pressed(MouseMiddle)) {
                self.world.run_workload(&self.controls[&Pressed(MouseMiddle)]);
            }
        }
        if input::is_mouse_button_pressed(ctx, MouseButton::Right) {
            if self.controls.contains_key(&Pressed(MouseRight)) {
                self.world.run_workload(&self.controls[&Pressed(MouseRight)]);
            }
        }
        if input::is_mouse_button_pressed(ctx, MouseButton::X1) {
            if self.controls.contains_key(&Pressed(MouseX1)) {
                self.world.run_workload(&self.controls[&Pressed(MouseX1)]);
            }
        }
        if input::is_mouse_button_pressed(ctx, MouseButton::X2) {
            if self.controls.contains_key(&Pressed(MouseX2)) {
                self.world.run_workload(&self.controls[&Pressed(MouseX2)]);
            }
        }
        if input::is_mouse_button_down(ctx, MouseButton::Left) {
            if self.controls.contains_key(&Held(MouseLeft)) {
                self.world.run_workload(&self.controls[&Held(MouseLeft)]);
            }
        }
        if input::is_mouse_button_down(ctx, MouseButton::Middle) {
            if self.controls.contains_key(&Held(MouseMiddle)) {
                self.world.run_workload(&self.controls[&Held(MouseMiddle)]);
            }
        }
        if input::is_mouse_button_down(ctx, MouseButton::Right) {
            if self.controls.contains_key(&Held(MouseRight)) {
                self.world.run_workload(&self.controls[&Held(MouseRight)]);
            }
        }
        if input::is_mouse_button_down(ctx, MouseButton::X1) {
            if self.controls.contains_key(&Held(MouseX1)) {
                self.world.run_workload(&self.controls[&Held(MouseX1)]);
            }
        }
        if input::is_mouse_button_down(ctx, MouseButton::X2) {
            if self.controls.contains_key(&Held(MouseX2)) {
                self.world.run_workload(&self.controls[&Held(MouseX2)]);
            }
        }
        if input::is_mouse_button_released(ctx, MouseButton::Left) {
            if self.controls.contains_key(&Released(MouseLeft)) {
                self.world.run_workload(&self.controls[&Released(MouseLeft)]);
            }
        }
        if input::is_mouse_button_released(ctx, MouseButton::Middle) {
            if self.controls.contains_key(&Released(MouseMiddle)) {
                self.world.run_workload(&self.controls[&Released(MouseMiddle)]);
            }
        }
        if input::is_mouse_button_released(ctx, MouseButton::Right) {
            if self.controls.contains_key(&Released(MouseRight)) {
                self.world.run_workload(&self.controls[&Released(MouseRight)]);
            }
        }
        if input::is_mouse_button_released(ctx, MouseButton::X1) {
            if self.controls.contains_key(&Released(MouseX1)) {
                self.world.run_workload(&self.controls[&Released(MouseX1)]);
            }
        }
        if input::is_mouse_button_released(ctx, MouseButton::X2) {
            if self.controls.contains_key(&Released(MouseX2)) {
                self.world.run_workload(&self.controls[&Released(MouseX2)]);
            }
        }
    }
}