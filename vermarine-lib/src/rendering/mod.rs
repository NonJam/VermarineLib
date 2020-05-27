use std::collections::{HashMap,};
use tetra::graphics::Texture;
use tetra::Context;
use std::path::Path;

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