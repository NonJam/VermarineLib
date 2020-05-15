use tetra::Context;
use tetra::graphics::Texture;
use std::collections::HashMap;
use glob::glob;


pub(crate) type Textures = HashMap<String, Texture>;

pub(crate) struct Resources {
    pub(crate) textures: Textures
}
impl Resources {
    pub(crate) fn load(ctx: &mut Context, path: &'static str) -> Self {

        let mut textures = Textures::new();
    
        let temp = [path, "/**/*.png"].join("");
        let pattern = temp.as_str();
    
        for entry in glob(pattern).expect("Failed to read glob pattern") {
            if let Ok(file) = entry {
                let name = file.file_stem().unwrap().to_str().unwrap().to_string();
                textures.insert(name, Texture::new(ctx, file).unwrap());
            }
        }

        Resources { textures }
    }
}