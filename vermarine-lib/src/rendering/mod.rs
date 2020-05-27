use std::collections::{HashMap,};
use tetra::{
    graphics::{
        Texture,
        DrawParams,
        Drawable,
    },
    math::{
        Vec2,
    },
    Context,
};
use std::path::Path;
use shipyard::{
    *,
};

use crate::{
    physics::{
        PhysicsBody,
        Transform,
        world::{
            PhysicsWorld,
        },
    },
};

pub trait Renderable {
    fn draw(&mut self, transform: &Transform, context: &mut Context, renderables: &Renderables, entity: EntityId);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sprite {
    pub renderable: &'static str,
    pub draw_params: DrawParams,
}

impl Sprite {
    pub fn new(renderable: &'static str, draw_params: DrawParams) -> Self {
        Sprite {
            renderable,
            draw_params,
        }
    }

    //pub fn from_renderable(renderable: &'static str) -> Self {
    //    Sprite {
    //        renderable,
    //        draw_params: DrawParams::new(),
    //    }
    //}
}

impl Renderable for Sprite {
    fn draw(&mut self, transform: &Transform, context: &mut Context, renderables: &Renderables, _: EntityId) {
        let texture = if let Some(texture) = renderables.lookup.get(self.renderable) {
            texture
        } else { return; };

        let position = self.draw_params.position + Vec2::new(transform.x as f32, transform.y as f32);
        let mut params = self.draw_params;
        params.position = position;

        texture.draw(context, params);
    }
}

pub fn render_renderables<T: Renderable + Send + Sync + 'static>(data: (&Renderables, &mut Context), bodies: View<PhysicsBody>, physics_world: UniqueViewMut<PhysicsWorld>, mut ts: ViewMut<T>) {
    let (renderables, context) = data;

    for (id, (_, renderable)) in (&bodies, &mut ts).iter().with_id() {
        let transform = physics_world.transform(id);
        renderable.draw(transform, context, renderables, id);
    }
}

pub struct Renderables {
    pub lookup: HashMap<&'static str, Texture>,
}

impl Renderables {
    pub fn new(ctx: &mut Context) -> tetra::Result<Self> {
        let mut map = HashMap::new();

        let pngs = get_textures(ctx, "assets/")?;

        for (key, value) in pngs.into_iter() {
            map.insert(key, value);
        }

        Ok(Renderables {
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