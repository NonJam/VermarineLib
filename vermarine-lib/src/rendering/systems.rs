use shipyard::{
    *,
};
use tetra::{
    math::{
        Vec3,
    },
};
use crate::{
    components::{
        Transform,
    },
    rendering::{
        Sprite,
        draw_buffer::{
            DrawBuffer,
        }
    },
};

/// Adds commands to DrawBuffer for all Sprite components
pub fn draw_sprites(sprites: View<Sprite>, mut draw_buffer: UniqueViewMut<DrawBuffer>, transforms: View<Transform>) {
    for (transform, sprite) in (&transforms, &sprites).iter() {
        let mut command = sprite.0;
        command.position = command.position + Vec3::new(transform.x as f32, transform.y as f32, 0.0);
        draw_buffer.draw(command);
    }
}