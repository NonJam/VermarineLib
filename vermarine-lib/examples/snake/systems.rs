use shipyard::*;
use rand::Rng;
use rand::rngs::StdRng;
use vermarine_lib::components::*;
use crate::components::*;

pub fn new_game(
    mut entities: EntitiesViewMut, 
    mut transforms: ViewMut<Transform>, 
    mut sprites: ViewMut<Sprite>,
    mut segments: ViewMut<Segment>,
    mut rng: UniqueViewMut<StdRng>,
) {
    entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
        Transform::new(432f64, 400f64, 16f64),
        Sprite::new("circle"),
        Segment { position: 0 }
    ));
    entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
        Transform::new(400f64, 400f64, 16f64),
        Sprite::new("circle"),
        Segment { position: 1 }
    ));
    entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
        Transform::new(368f64, 400f64, 16f64),
        Sprite::new("circle"),
        Segment { position: 2 }
    ));

    let mut randx = 16f64 + (rng.gen_range(0,25) * 32) as f64;
    let mut randy = 16f64 + (rng.gen_range(0,25) * 32) as f64;

    while randy == 400f64 && (randx == 368f64 || randx == 400f64 || randx == 432f64) {
        randx = 16f64 + (rng.gen_range(0,25) * 32) as f64;
        randy = 16f64 + (rng.gen_range(0,25) * 32) as f64;
    }

    entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
        Transform::new(randx, randy, 16f64),
        Sprite::new("circle"),
        Segment { position: -1 }
    ));
}

pub fn move_snake(
    mut entities: EntitiesViewMut, 
    mut snake: UniqueViewMut<SnakeGame>, 
    mut transforms: ViewMut<Transform>,
    mut sprites: ViewMut<Sprite>,
    mut segments: ViewMut<Segment>,
    mut rng: UniqueViewMut<StdRng>,
) {
    if snake.frame_counter < snake.skip_frames {
        snake.frame_counter += 1;
    } else {
        snake.frame_counter = 0;
        
        let mut head_x = 0f64;
        let mut head_y = 0f64;
        let mut head = vec![];
        let mut pickup = vec![];
        let mut all_x = vec![];
        let mut all_y = vec![];
        let mut all_segments = vec![];

        for (transform, segment) in (&mut transforms, &mut segments).iter() {

            all_x.push(transform.x.clone());
            all_y.push(transform.y.clone());

            if segment.position == 0 {
                head_x = transform.x;
                head_y = transform.y;
            }

            if segment.position >= 0 {
                segment.position += 1;
            }

            if segment.position == snake.length {
                segment.position = 0;
                head.push(transform);
            }
            else if segment.position == -1 {
                pickup.push(transform);
            }

            all_segments.push(segment);
        }


        let new_x = head_x + snake.move_x;
        let new_y = head_y + snake.move_y;

        if new_x == pickup[0].x && new_y == pickup[0].y {

            snake.length += 1;

            for segment in all_segments.into_iter() {
                if segment.position == 0 {
                    segment.position = snake.length - 1;
                }
                if segment.position == -1 {
                    segment.position = 0;
                }
            }

            let mut randx = 16f64 + (rng.gen_range(0,25) * 32) as f64;
            let mut randy = 16f64 + (rng.gen_range(0,25) * 32) as f64;

            while all_x.contains(&randx) {
                randx = 16f64 + (rng.gen_range(0,25) * 32) as f64;
            }
            
            while all_y.contains(&randy) {
                randy = 16f64 + (rng.gen_range(0,25) * 32) as f64;
            }
        
            entities.add_entity((&mut transforms, &mut sprites, &mut segments), (
                Transform::new(randx, randy, 16f64),
                Sprite::new("circle"),
                Segment { position: -1 }
            ));

        } else {

            head[0].x = head_x + snake.move_x;
            head[0].y = head_y + snake.move_y;
    
            snake.move_x_prev = snake.move_x;
            snake.move_y_prev = snake.move_y;
        }
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