use shipyard::*;
use vermarine_lib::components::*;
use crate::components::*;

pub fn new_game(
    mut entities: EntitiesViewMut, 
    mut transforms: ViewMut<Transform>, 
    mut sprites: ViewMut<Sprite>,
    mut segments: ViewMut<Segment>,
) {
    entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
        Transform::new(640f64, 360f64, 16f64),
        Sprite::new("circle"),
        Segment { position: 0 }
    ));
    entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
        Transform::new(640f64 - 32f64, 360f64, 16f64),
        Sprite::new("circle"),
        Segment { position: 1 }
    ));
    entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
        Transform::new(640f64 - 64f64, 360f64, 16f64),
        Sprite::new("circle"),
        Segment { position: 2 }
    ));
}

pub fn move_snake(
    mut snake: UniqueViewMut<SnakeGame>, 
    mut transforms: ViewMut<Transform>,
    mut segments: ViewMut<Segment>,
) {
    if snake.frame_counter < snake.skip_frames {
        snake.frame_counter += 1;
    } else {
        snake.frame_counter = 0;
        
        let mut head_x = 0f64;
        let mut head_y = 0f64;
        let mut head = vec![];

        for (transform, segment) in (&mut transforms, &mut segments).iter() {
            
            if segment.position == 0 {
                head_x = transform.x;
                head_y = transform.y;
            }
            
            segment.position += 1;

            if segment.position == snake.length {
                segment.position = 0;
                head.push(transform);
            }
        }

        head[0].x = head_x + snake.move_x;
        head[0].y = head_y + snake.move_y;

        snake.move_x_prev = snake.move_x;
        snake.move_y_prev = snake.move_y;
    }
}

pub fn move_up(mut snake: UniqueViewMut<SnakeGame>) {
    if snake.move_y_prev != 32f64 {
        snake.move_y = -32f64;
        snake.move_x = 0f64;
    }
}

pub fn move_down(mut snake: UniqueViewMut<SnakeGame>) {
    if snake.move_y_prev != -32f64 {
        snake.move_y = 32f64;
        snake.move_x = 0f64;
    }
}

pub fn move_left(mut snake: UniqueViewMut<SnakeGame>) {
    if snake.move_x_prev != 32f64 {
        snake.move_x = -32f64;
        snake.move_y = 0f64;
    }
}

pub fn move_right(mut snake: UniqueViewMut<SnakeGame>) {
    if snake.move_x_prev != -32f64 {
        snake.move_x = 32f64;
        snake.move_y = 0f64;
    }
}