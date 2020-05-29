pub mod draw_buffer;
pub mod systems;

use std::collections::HashMap;
use tetra::{
    graphics::{
        Texture,
        DrawParams,
    },
    Context,
};
use draw_buffer::{
    DrawCommand,
    DrawBuffer,
};
use shipyard::*;
use std::path::Path;

/// Dummy trait to allow adding a method to World
pub trait RenderingWorkloadCreator {
    fn add_rendering_workload(&mut self) -> WorkloadBuilder;
}

impl RenderingWorkloadCreator for World {
    fn add_rendering_workload(&mut self) -> WorkloadBuilder {
        self.add_unique(DrawBuffer::new());
        self.add_workload("Rendering")
    }
}

/// Dummy trait to allow adding a method to WorkloadBuilder
pub trait RenderingWorkloadSystems<'a> {
    fn with_rendering_systems(self) -> WorkloadBuilder<'a>;
}

impl<'a> RenderingWorkloadSystems<'a> for WorkloadBuilder<'a> {
    fn with_rendering_systems(self) -> WorkloadBuilder<'a> {
        self
            .with_system(system!(systems::draw_sprites))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sprite(pub DrawCommand);

impl Sprite {
    pub fn new(drawable: &'static str) -> Self {
        Sprite(DrawCommand::new(drawable))
    }

    pub fn from_command(draw_command: DrawCommand) -> Self {
        Sprite(draw_command)
    }
}

pub struct Drawables {
    pub lookup: HashMap<&'static str, Texture>,
}

impl Drawables {
    pub fn new(ctx: &mut Context) -> tetra::Result<Self> {
        let mut map = HashMap::new();

        let pngs = get_textures(ctx, "assets/")?;

        for (key, value) in pngs.into_iter() {
            map.insert(key, value);
        }

        Ok(Drawables {
            lookup: map,
        })
    }
}

pub fn get_textures<P: AsRef<Path>>(ctx: &mut Context, dir: P) -> tetra::Result<Vec<(&'static str, Texture)>> {
    use std::fs::read_dir;

    let mut found = vec![];

    for file in read_dir(dir).unwrap() {
        let file = file.unwrap();
        if file.file_type().unwrap().is_file() {
            let path = file.path();
            if let Some(ext) = path.extension() {
                if ext == "png" {
                    if let Some(stem) = path.file_stem() {
                        let stem = stem.to_string_lossy().into_owned();
                        let texture = Texture::new(ctx, path);

                        fn to_str(string: String) -> &'static str {
                            Box::leak(string.into_boxed_str())
                        }
                        let foo = to_str(stem);

                        found.push((foo, texture?));
                    }
                }
            }
        } else {
            found.append(&mut get_textures(ctx, file.path())?);
        }
    }

    Ok(found)
}