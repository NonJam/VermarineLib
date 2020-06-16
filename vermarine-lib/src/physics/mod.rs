pub mod world;
pub mod spatialhash;
pub mod sat;

use crate::components::Transform;
use shipyard::*;
use tetra::math::Vec2;
use world::*;
use spatialhash::*;

/// Dummy trait to allow adding a method to World
pub trait PhysicsWorkloadCreator {
    fn add_physics_workload(&mut self, bucket_width: f64, bucket_height: f64) -> WorkloadBuilder;
}

impl PhysicsWorkloadCreator for shipyard::World {
    fn add_physics_workload(&mut self, bucket_width: f64, bucket_height: f64) -> WorkloadBuilder {
        self.add_unique(PhysicsWorld::new(bucket_width, bucket_height));
        self.borrow::<ViewMut<PhysicsBody>>().update_pack();
        self.add_workload("Physics")
    }
}

/// Dummy trait to allow adding a method to WorkloadBuilder
pub trait PhysicsWorkloadSystems<'a> {
    fn with_physics_systems(self) -> WorkloadBuilder<'a>;
}

impl<'a> PhysicsWorkloadSystems<'a> for WorkloadBuilder<'a> {
    fn with_physics_systems(self) -> WorkloadBuilder<'a> {
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
}

}

#[derive(Default)]
pub struct PhysicsBody;

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

    pub normal: Vec2<f64>,
}

impl Collision {
    pub fn new(transform1: Transform, shape1: CollisionShape, collides_with1: u64, collision_layer1: u64,
        transform2: Transform, shape2: CollisionShape, collides_with2: u64, collision_layer2: u64, entity2: EntityId, normal: Vec2<f64>) -> Self {
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

            normal,
        }
    }
}

#[derive(Clone, Default)]
pub struct CollisionBody {
    pub colliders: Vec<Collider>,
    pub sensors: Vec<Collider>,
    aabb: AABB,
}

impl CollisionBody {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_collider(collider: Collider) -> Self {
        let aabb = AABB::from_collider(&collider);
        CollisionBody {
            colliders: vec![collider],
            sensors: vec![],
            aabb,
        }
    }

    pub fn from_colliders(colliders: Vec<Collider>) -> Self {
        let aabb = AABB::from_colliders(&colliders);
        CollisionBody {
            colliders,
            sensors: vec![],
            aabb,
        }
    }

    pub fn from_sensor(sensor: Collider) -> Self {
        let aabb = AABB::from_collider(&sensor);
        CollisionBody {
            colliders: vec![],
            sensors: vec![sensor],
            aabb,
        }
    }

    pub fn from_sensors(sensors: Vec<Collider>) -> Self {
        let aabb = AABB::from_colliders(&sensors);
        CollisionBody {
            colliders: vec![],
            sensors,
            aabb,
        }
    }

    pub fn from_parts(colliders: Vec<Collider>, sensors: Vec<Collider>) -> Self {
        let mut joined = colliders.clone();
        joined.append(&mut sensors.clone());
        let aabb = AABB::from_colliders(&joined);
        CollisionBody {
            colliders,
            sensors,
            aabb,
        }
    }

    pub fn from_body(body: &CollisionBody) -> Self {
        let mut new_body = CollisionBody::new();

        for collider in body.colliders.iter() {
            new_body.colliders.push(
                Collider::from_collider(&collider)
            );
        }

        for sensor in body.sensors.iter() {
            new_body.sensors.push(
                Collider::from_collider(&sensor)
            );
        }

        new_body
    }

    pub(crate) fn remove_collision(&mut self, entity: EntityId) {
        for collider in self.colliders.iter_mut() {
            let mut counter = 0;
            loop {
                let collision = collider.overlapping.get(counter);

                if let Some(collision) = collision {
                    if collision.entity2 == entity {
                        collider.overlapping.remove(counter);
                    } else {
                        counter += 1;
                    }
                } else {
                    break;
                }
            }
        }

        for collider in self.sensors.iter_mut() {
            let mut counter = 0;
            loop {
                let collision = collider.overlapping.get(counter);

                if let Some(collision) = collision {
                    if collision.entity2 == entity {
                        collider.overlapping.remove(counter);
                    } else {
                        counter += 1;
                    }
                } else {
                    break;
                }
            }
        }
    }

    pub(crate) fn remove_all_collisions(&mut self) {
        for collider in self.colliders.iter_mut() {
            collider.overlapping.clear();
        }

        for collider in self.sensors.iter_mut() {
            collider.overlapping.clear();
        }
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

    pub fn half_extents(width: f64, height: f64, collision_layer: u64, collides_with: u64) -> Self {
        let vertices = vec![
            Vec2::new(-width, -height),
            Vec2::new(width, -height),
            Vec2::new(width, height),
            Vec2::new(-width, height),
        ];

        Collider {
            shape: CollisionShape::Polygon(vertices),
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

#[derive(Clone, Default, Debug)]
pub struct AABB {
    pub dx: f64,
    pub dy: f64,
    pub width: f64,
    pub height: f64,
}

impl AABB {
    pub fn new(dx: f64, dy: f64, width: f64, height: f64) -> Self {
        AABB {
            dx,
            dy,
            width,
            height,
        }
    }

    pub fn from_collider(collider: &Collider) -> Self {
        Self::from_colliders(&vec![collider.clone()])
    }

    pub fn from_colliders(colliders: &Vec<Collider>) -> Self {
        let mut xmin = None;
        let mut xmax = None;
        let mut ymin = None;
        let mut ymax = None;

        use CollisionShape::*;
        
        for collider in colliders.iter() {
            match &collider.shape {
                Polygon(vertices) => { 
                    for vertex in vertices.iter() {
                        if xmin.is_none() || vertex.x < xmin.unwrap() {
                            xmin = Some(vertex.x);
                        }
                        if xmax.is_none() || vertex.x > xmax.unwrap() {
                            xmax = Some(vertex.x);
                        }
                        if ymin.is_none() || vertex.y < ymin.unwrap() {
                            ymin = Some(vertex.y);
                        }
                        if ymax.is_none() || vertex.y > ymin.unwrap() {
                            ymax = Some(vertex.y);
                        }
                    }
                },
                Circle(r) => { 
                    let r = *r;
                    if xmin.is_none() || -r < xmin.unwrap() {
                        xmin = Some(-r);
                    }
                    if xmax.is_none() || r > xmax.unwrap() {
                        xmax = Some(r);
                    }
                    if ymin.is_none() || -r < ymin.unwrap() {
                        ymin = Some(-r);
                    }
                    if ymax.is_none() || r > ymin.unwrap() {
                        ymax = Some(r);
                    }
                },
            };
        }

        let xmin = xmin.unwrap();
        let xmax = xmax.unwrap();
        let ymin = ymin.unwrap();
        let ymax = ymax.unwrap();

        let aabb = AABB {
            dx: if xmin < 0.0 { xmin } else { 0.0 },
            dy: if ymin < 0.0 { ymin } else { 0.0 },
            width: xmax - xmin,
            height: ymax - ymin,
        };
        aabb
    }
}

#[derive(Clone)]
pub enum CollisionShape {
    Circle(f64),
    Polygon(Vec<Vec2<f64>>)
}

impl CollisionShape {
    pub fn is_circle(&self) -> bool {
        match self {
            Self::Circle(_) => true,
            _ => false,
        }
    }

    pub fn get_width(&self) -> f64 {
        match self {
            Self::Circle(r) => r * 2.0,
            Self::Polygon(vertices) => {
                let mut leftest = None;
                let mut rightest = None;

                for vertex in vertices.iter() {
                    if let Some(x) = leftest {
                        if vertex.x <= x {
                            leftest = Some(vertex.x);
                        }
                    } else {
                        leftest = Some(vertex.x);
                    }

                    if let Some(x) = rightest {
                        if vertex.x >= x {
                            rightest = Some(vertex.x);
                        }
                    } else {
                        rightest = Some(vertex.x);
                    }
                }

                rightest.unwrap() - leftest.unwrap()
            }
        }
    }
}

//
//

pub fn split_around_index<T>(slice: &[T], index: usize) -> (&[T], &T, &[T]) {
    let (left, right) = slice.split_at(index);
    let (middle, right) = right.split_first().unwrap();
    (left, middle, right)
}

pub fn split_around_index_mut<T>(slice: &mut [T], index: usize) -> (&mut [T], &mut T, &mut [T]) {
    let (left, right) = slice.split_at_mut(index);
    let (middle, right) = right.split_first_mut().unwrap();
    (left, middle, right)
}

//
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bug_reproduction() {
        let mut world = World::new();
        
        world
            .add_physics_workload(50.0, 50.0)
            .with_physics_systems()
            .build();

        // Setup
        let e1 = world.run(|
            mut entities: EntitiesViewMut,
            mut bodies: ViewMut<PhysicsBody>,
            mut physics_world: UniqueViewMut<PhysicsWorld>| { 
                let e1 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    e1, 
                    Transform::new(10.0, 10.0), 
                    CollisionBody::from_collider(Collider::half_extents(2.0, 2.0, 1, 2)),
                );

                let e2 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    e2, 
                    Transform::new(0.0, 0.0), 
                    CollisionBody::from_collider(Collider::half_extents(2.0, 2.0, 2, 1)),
                );
                
                // Move e2 into e1
                physics_world.move_body(e2, Vec2::new(10.0, 10.0));

                assert_eq!(physics_world.collider(e2).colliders[0].overlapping.len(), 1);
                assert_eq!(physics_world.collider(e2).colliders[0].overlapping[0].entity2, e1);

                assert_eq!(physics_world.collider(e1).colliders[0].overlapping.len(), 1);
                assert_eq!(physics_world.collider(e1).colliders[0].overlapping[0].entity2, e2);

                e1
        });

        // Kill entity
        world.run(|mut all_storages: AllStoragesViewMut| {
            let mut to_kill = None;
            {
                let physics_world = all_storages.borrow::<UniqueViewMut<PhysicsWorld>>();

                // Check for collision
                let collision_body = physics_world.collider(e1);
                
                assert_eq!(collision_body.colliders[0].overlapping.len(), 1);

                for collision in collision_body.colliders[0].overlapping.iter() {
                    to_kill = Some(collision.entity2);
                }
            }

            all_storages.delete(to_kill.unwrap());
            let (mut world, mut bodies) = all_storages.borrow::<(UniqueViewMut<PhysicsWorld>, ViewMut<PhysicsBody>)>();
            world.sync(&mut bodies);

            let collision_body = world.collider(e1);
            assert_eq!(collision_body.colliders[0].overlapping.len(), 0);
        });

        // Move alive entity
        world.run(|
            mut world: UniqueViewMut<PhysicsWorld>,| {
                world.move_body(e1, Vec2::new(-10.0, -10.0));
                assert_eq!(world.collider(e1).colliders[0].overlapping.len(), 0);
        });
    }

    #[test]
    fn bug_two() {
        let mut world = World::new();

        world
            .add_physics_workload(50.0, 50.0)
            .with_physics_systems()
            .build();

        // Setup
        let e1 = world.run(|
            mut entities: EntitiesViewMut,
            mut bodies: ViewMut<PhysicsBody>,
            mut physics_world: UniqueViewMut<PhysicsWorld>| { 
                let e1 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    e1, 
                    Transform::new(10.0, 10.0), 
                    CollisionBody::from_sensor(Collider::half_extents(2.0, 2.0, 1, 2)),
                );

                (0..100).for_each(|_| { entities.add_entity((), ()); });

                let e2 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    e2, 
                    Transform::new(0.0, 0.0), 
                    CollisionBody::from_collider(Collider::half_extents(2.0, 2.0, 2, 1)),
                );
                
                println!("E1: {:?}\nE2: {:?}\n", e1, e2);

                // Move e2 into e1
                physics_world.move_body(e2, Vec2::new(10.0, 10.0));

                assert_eq!(physics_world.collider(e2).colliders[0].overlapping.len(), 0);

                assert_eq!(physics_world.collider(e1).sensors[0].overlapping.len(), 1);
                assert_eq!(physics_world.collider(e1).sensors[0].overlapping[0].entity2, e2);

                e1
        });

        // Kill entity
        world.run(|mut all_storages: AllStoragesViewMut| {
            let mut to_kill = None;
            {
                let physics_world = all_storages.borrow::<UniqueViewMut<PhysicsWorld>>();

                // Check for collision
                let collision_body = physics_world.collider(e1);
                
                assert_eq!(collision_body.sensors[0].overlapping.len(), 1);

                for collision in collision_body.sensors[0].overlapping.iter() {
                    to_kill = Some(collision.entity2);
                }
            }

            all_storages.delete(to_kill.unwrap());
            let (mut world, mut bodies) = all_storages.borrow::<(UniqueViewMut<PhysicsWorld>, ViewMut<PhysicsBody>)>();
            world.sync(&mut bodies);

            let collision_body = world.collider(e1);
            assert_eq!(collision_body.sensors[0].overlapping.len(), 0);
        });

        // Move alive entity
        world.run(|
            mut world: UniqueViewMut<PhysicsWorld>,| {
                world.move_body(e1, Vec2::new(-10.0, -10.0));
                assert_eq!(world.collider(e1).sensors[0].overlapping.len(), 0);
        });
    }
}