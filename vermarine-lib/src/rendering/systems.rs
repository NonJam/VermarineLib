use shipyard::{
    *,
};
use tetra::{
    math::{
        Vec3,
    },
};
use crate::{
    physics::{
        PhysicsBody,
        world::{
            PhysicsWorld,
        },
    },
    rendering::{
        Sprite,
        draw_buffer::{
            DrawBuffer,
        }
    },
};

/// Adds commands to DrawBuffer for all Sprite components
pub fn draw_sprites(sprites: View<Sprite>, mut draw_buffer: UniqueViewMut<DrawBuffer>, bodies: View<PhysicsBody>, world: UniqueView<PhysicsWorld>) {
    for (id, (_, sprite)) in (&bodies, &sprites).iter().with_id() {
        let transform = world.transform(id);
        let mut command = sprite.0;
        command.position = command.position + Vec3::new(transform.x as f32, transform.y as f32, 0.0);
        draw_buffer.draw(command);
    }
}