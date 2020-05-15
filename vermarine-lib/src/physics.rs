use shipyard::*;

use crate::components::Transform;

pub fn physics_workload(world: &mut World) -> &'static str {
    let name = "Physics";
    
    world.add_workload(name)
        .with_system(system!(clear_collisions))
        .with_system(system!(calc_collisions))
        .build();

    name
}

pub fn clear_collisions(mut collision_bodies: ViewMut<CollisionBody>) {
    for body in (&mut collision_bodies).iter() {
        for collider in body.colliders.iter_mut() {
            collider.overlapping.clear();
        }
    }
}

pub fn calc_collisions(transforms: View<Transform>, mut colliders: ViewMut<CollisionBody>) {
    let mut collected: Vec<(EntityId, (&Transform, &mut CollisionBody))> = (&transforms, &mut colliders).iter().with_id().collect();

    let len = collected.len();

    for i in 1..len {
        let (left, right) = collected.split_at_mut(i);
        let body1 = left.last_mut().unwrap();
        for body2 in right.iter_mut() {
            let c1 = &mut (body1.1).1.colliders[0];
            let c2 = &mut (body2.1).1.colliders[0];
            
            if collider_overlaps_collider((body1.1).0, c1, (body2.1).0, c2) {                
                if c1.collides_with & c2.collision_layer > 0 {
                    let collision = Collision::new((body1.1).0.clone(), c1.shape.clone(), c1.collides_with, c1.collision_layer,
                        (body2.1).0.clone(), c2.shape.clone(), c2.collides_with, c2.collision_layer, body2.0);

                    c1.overlapping.push(collision.clone());
                }

                if c2.collides_with & c1.collision_layer > 0 {
                    let collision = Collision::new((body2.1).0.clone(), c2.shape.clone(), c2.collides_with, c2.collision_layer,
                        (body1.1).0.clone(), c1.shape.clone(), c1.collides_with, c1.collision_layer, body1.0);

                    c2.overlapping.push(collision.clone());
                }
            }
        }
    }
}

pub struct PhysicsWorld {
    bodies: Vec<CollisionBody>,
    transforms: Vec<Transform>,
}

impl PhysicsWorld {
    pub fn move_and_collide(id: EntityId, ) -> Option<Collision> {
        

        None
    }
}

fn blah() {
    let mut blah = PhysicsBody {
        dx: 0.0,
        dy: 0.0,
        handler: vec![],
    };

    let to_add = 20.0;
    let delta = 10.0;

    blah.move_delta(delta, 15.0, Box::from(move |body: &mut PhysicsBody| { body.dx += to_add; }));

    let blah2 = blah.handler.pop().unwrap();
    blah2(&mut blah);

    println!("{} == {}", blah.dx, to_add + delta);
}

pub struct PhysicsBody {
    dx: f64,
    dy: f64,
    handler: Vec<Box<dyn Fn(&mut PhysicsBody) -> ()>>,
}

impl PhysicsBody {
    pub fn new() -> Self {
        PhysicsBody {
            dx: 0f64,
            dy: 0f64,
            handler: vec![Box::new(|_| { } )],
        }
    }

    pub fn move_delta(&mut self, x: f64, y: f64, handler: Box<dyn Fn(&mut PhysicsBody) -> ()>) {
        self.dx += x;
        self.dy += y;
        self.handler.push(handler);
    }
}

#[derive(Clone)]
pub struct Collision {
    pub transform1: Transform,
    pub shape1: CollisionShape,
    pub collides_with1: u64,
    pub collision_layer1: u64,

    pub transform2: Transform,
    pub shape2: CollisionShape,
    pub collides_with2: u64,
    pub collision_layer2: u64,
    pub entity2: EntityId,
}

impl Collision {
    pub fn new(transform1: Transform, shape1: CollisionShape, collides_with1: u64, collision_layer1: u64,
        transform2: Transform, shape2: CollisionShape, collides_with2: u64, collision_layer2: u64, entity2: EntityId) -> Self {
        Collision {
            transform1,
            shape1,
            collides_with1,
            collision_layer1,

            transform2,
            shape2,
            collides_with2,
            collision_layer2,
            entity2,
        }
    }
}

#[derive(Clone)]
pub struct CollisionBody {
    pub colliders: Vec<Collider>,
}

impl CollisionBody {
    pub fn new(collider: Collider) -> Self {
        CollisionBody {
            colliders: vec![collider],
        }
    }

    pub fn from_body(body: &CollisionBody) -> Self {
        let mut new_body = CollisionBody { colliders: vec![] };

        for collider in body.colliders.iter() {
            new_body.colliders.push(
                Collider::from_collider(&collider)
            );
        }

        new_body
    }
}

#[derive(Clone)]
pub struct Collider {
    pub shape: CollisionShape,
    pub collision_layer: u64,
    pub collides_with: u64,

    pub overlapping: Vec<Collision>,
}

impl Collider {
    pub fn circle(radius: f64, collision_layer: u64, collides_with: u64) -> Self {
        Collider {
            shape: CollisionShape::Circle(radius),
            collides_with,
            collision_layer,

            overlapping: vec![],
        }
    }

    pub fn from_collider(collider: &Collider) -> Self {
        Collider {
            shape: collider.shape.clone(),
            collision_layer: collider.collision_layer.clone(),
            collides_with: collider.collides_with.clone(),

            overlapping: vec![],
        }
    }
}

#[derive(Clone)]
pub enum CollisionShape {
    Circle(f64),
    //Composite(Vec<CollisionShape>),
}

pub fn collider_overlaps_collider(t1: &Transform, c1: &Collider, t2: &Transform, c2: &Collider) -> bool {
    match c1.shape {
        CollisionShape::Circle(r) => circle_overlaps_collider(t1, r, t2, c2),
    }
}

pub fn circle_overlaps_collider(t1: &Transform, c1: f64, t2: &Transform, c2: &Collider) -> bool {
    match c2.shape {
        CollisionShape::Circle(r) => circle_overlaps_circle(t1, c1, t2, r),
    }
}

pub fn circle_overlaps_circle(t1: &Transform, c1: f64, t2: &Transform, c2: f64) -> bool {
    let x = (t1.x - t2.x).abs();
    let y = (t1.y - t2.y).abs();
    x * x + y * y <= (c1 + c2) * (c1 + c2)
}