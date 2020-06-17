use tetra::{
    graphics::{
        self,
        Drawable,
        Color,
        Camera,
    },
    Context,
    math::{
        Vec2,
        Vec3,
    },
};
use super::{
    DrawParams,
    Drawables,
};
use std::cmp::Ordering;
use shipyard::{
    self,
    *,
};

pub struct DrawBuffer {
    commands: Vec<DrawCommand>,
}

impl DrawBuffer {
    pub fn new() -> Self {
        DrawBuffer {
            commands: vec![],
        }
    }

    pub fn draw(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn flush(data: (&mut Context, &Drawables), mut draw_buffer: UniqueViewMut<DrawBuffer>, mut camera: UniqueViewMut<Camera>) {
        let (ctx, drawables) = data;

        draw_buffer.sort();

        camera.update();
        graphics::set_transform_matrix(ctx, camera.as_matrix());
        
        for cmd in draw_buffer.commands.iter_mut() {
            let drawable = drawables.lookup.get(cmd.drawable).unwrap();

            let mut params = DrawParams::new()
                .position(Vec2::new(cmd.position.x, cmd.position.y))
                .scale(cmd.scale)
                .origin(cmd.origin)
                .rotation(cmd.rotation)
                .color(cmd.color);

            if cmd.draw_iso == true {
                params.position.y -= cmd.position.z;
            }

            drawable.draw(ctx, params);
        }

        draw_buffer.commands.clear();
    }

    pub fn debug_command_buffer(&self) {
        let mut output: String = "\n\n\n\n\n START: \n\n".into();
        for elem in self.commands.iter() {
            output = format!("{}\n(x: {}, y: {}, z: {}, dl: {})", output, elem.position.x, elem.position.y, elem.position.z, elem.draw_layer);
        }
        output = format!("{}\n\n\n\n END \n\n\n\n", output);
        println!("{}", output);
    }

    /// This method is called automatically at the start of flush() 
    pub fn sort(&mut self) {
        self.commands.sort_by(|a, b| {
            if a.position.z == b.position.z {
                if a.draw_layer == b.draw_layer {
                    if a.position.y == b.position.y {
                        if a.position.x == b.position.x {
                            Ordering::Equal
                        } else {
                            a.position.x.partial_cmp(&b.position.x).unwrap()
                        }
                    } else {
                        a.position.y.partial_cmp(&b.position.y).unwrap()
                    }
                } else {
                    a.draw_layer.partial_cmp(&b.draw_layer).unwrap()
                }
            } else {
                a.position.z.partial_cmp(&b.position.z).unwrap()
            }
        });
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawCommand {
    /// The ID for a drawable entry in ``rendering::Renderables``
    pub drawable: &'static str,

    /// The position that the drawable should be drawn at. Defaults to `(0.0, 0.0, 0.0)`.
    /// Z-axis is used for draw order sorting and in the case of isometric is subtracted from the y-axis when drawn
    /// 
    /// When used in a Sprite the xy of position is used is an offset to the entities Transform
    pub position: Vec3<f32>,

    /// Used in draw order sorting
    pub draw_layer: f32,

    /// The scale that the graphic should be drawn at. Defaults to `(1.0, 1.0)`.
    ///
    /// This can be set to a negative value to flip the graphic around the origin.
    pub scale: Vec2<f32>,

    /// The origin of the graphic. Defaults to `(0.0, 0.0)` (the top left).
    ///
    /// This offset is applied before scaling, rotation and positioning. For example, if you have
    /// a 16x16 image and set the origin to [8.0, 8.0], subsequent transformations will be performed
    /// relative to the center of the image.
    pub origin: Vec2<f32>,

    /// The rotation of the graphic, in radians. Defaults to `0.0`.
    pub rotation: f32,

    /// A color to multiply the graphic by. Defaults to `Color::WHITE`.
    pub color: Color,

    /// Flag to determine whether to use the Z component of position as an offset for the Y axis after sorting. 
    pub draw_iso: bool,
}

impl DrawCommand {
    pub fn new(drawable: &'static str) -> Self {
        DrawCommand {
            drawable,
            position: Vec3::default(),
            draw_layer: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::default(),
            rotation: 0.0,
            color: Color::WHITE,
            draw_iso: false,
        }
    }

    /// Sets the position that the graphic should be drawn at.
    pub fn position(mut self, position: Vec3<f32>) -> DrawCommand {
        self.position = position;
        self
    }

    /// Sets the draw layer
    pub fn draw_layer(mut self, draw_layer: f32) -> DrawCommand {
        self.draw_layer = draw_layer;
        self
    }

    /// Sets the scale that the graphic should be drawn at.
    pub fn scale(mut self, scale: Vec2<f32>) -> DrawCommand {
        self.scale = scale;
        self
    }

    /// Sets the origin of the graphic.
    pub fn origin(mut self, origin: Vec2<f32>) -> DrawCommand {
        self.origin = origin;
        self
    }

    /// Sets the rotation of the graphic, in radians.
    pub fn rotation(mut self, rotation: f32) -> DrawCommand {
        self.rotation = rotation;
        self
    }

    /// Sets the color to multiply the graphic by.
    pub fn color(mut self, color: Color) -> DrawCommand {
        self.color = color;
        self
    }

    /// Sets the draw_iso flag
    pub fn draw_iso(mut self, draw_iso: bool) -> DrawCommand {
        self.draw_iso = draw_iso;
        self
    }
}