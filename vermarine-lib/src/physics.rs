use shipyard::*;
use tetra::math::Vec2;

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
            
            let (collided, _mtv) = sat::seperating_axis_test((body1.1).0, &c1.shape, (body2.1).0, &c2.shape);
            if collided {
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
    transforms: Vec<Transform>,
    bodies: Vec<CollisionBody>,
    sparse: Vec<usize>,
}

impl PhysicsWorld {
    pub fn move_and_collide(&mut self, id: EntityId) -> Option<Collision> {
        

        None
    }

    pub fn data_from_id(&mut self, id: EntityId) -> Option<(&mut Transform, &mut CollisionBody)> {
        let index = id.uindex();
        if let Some(index) = self.sparse.get_mut(index) {
            let transform = self.transforms.get_mut(*index).unwrap();
            let body = self.bodies.get_mut(*index).unwrap();
            return Some((transform, body));
        }
        None
    }
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
}

mod sat {
    use super::*;

    pub fn get_axes(shape: &CollisionShape) -> Vec<Vec2<f64>> {
        use CollisionShape::Polygon;
        use CollisionShape::Circle;

        match shape {
            Polygon(vertices) => {
                // Get the normals of each edge of the polygon
                let mut axes1 = vec![];
                for i in 0..(vertices.len() - 1) {
                    let p1 = vertices[i];
                    let p2 = vertices[i + 1];
                    let edge = p1 - p2;
                    let normal = Vec2::new(edge.y, -edge.x);
                    axes1.push(normal);
                }
                axes1
            },
            Circle(_) => {
                // Circles dont have vertices so we can't calculate any normals here, get_circle_polygon_axis handles this.
                vec![]
            },
        }
    }

    pub fn get_circle_polygon_axis(circle: &CollisionShape, t1: &Transform, polygon: &CollisionShape, t2: &Transform) -> Vec2<f64> {
        use CollisionShape::Polygon;
        use CollisionShape::Circle;

        // Returns a vector from the vertex to the circle 
        fn get_axis(circle_pos: &Vec2<f64>, vertex: &Vec2<f64>, vertex_pos: &Transform) -> Vec2<f64> {
            let mut vertex = *vertex;
            vertex.x += vertex_pos.x;
            vertex.y += vertex_pos.y;

            circle_pos - vertex
        }

        if let (Circle(_), Polygon(vertices)) = (circle, polygon) {
            let circle_pos = Vec2::new(t1.x, t1.y);
            
            let start_axis = get_axis(&circle_pos, &vertices[0], t2);
            let mut smallest: f64 = start_axis.magnitude_squared(); 
            let mut axis: Vec2<f64> = start_axis;

            // Get the vertex closest to the circle
            for vertex in vertices.iter() {
                let found_axis = get_axis(&circle_pos, vertex, t2);

                if found_axis.magnitude_squared() < smallest {
                    smallest = found_axis.magnitude_squared();
                    axis = found_axis;
                }
            }

            return axis;
        }

        panic!("get_circle_polygon_axes() with incorrect collider shape arguments");
    }

    pub struct Projection {
        pub min: f64,
        pub max: f64,
    }

    impl Projection {
        pub fn new(min: f64, max: f64) -> Self {
            Projection {
                min,
                max,
            }
        }

        pub fn overlaps(&self, other: &Projection) -> bool {
            if self.min >= other.min && self.min <= other.max {
                return true;
            } 
            else if self.max >= other.min && self.max <= other.max {
                return true;
            } 
            else if self.max >= other.max && self.min <= other.min {
                return true;
            }

            return false;
        }

        pub fn get_overlap(&self, other: &Projection) -> f64 {
            if self.min >= other.min && self.max <= other.max  {
                return self.max - self.min;
            }
            else if self.max >= other.min && self.max <= other.max {
                return self.max - other.min;
            }
            else if self.min >= other.min && self.min <= other.max {
                return other.max - self.min;
            }

            0.0
        }
    }

    pub fn project_shape(shape: &CollisionShape, transform: &Transform, axis: &Vec2<f64>) -> Projection {
        use CollisionShape::Polygon;
        use CollisionShape::Circle;

        let pos = Vec2::new(transform.x, transform.y);

        match shape {
            Polygon(vertices) => {
                // Get the vertex with the highest dot product with axis
                // also get the vertex with the lowest dot product with axis
                let mut projection = Projection::new(axis.dot(vertices[0] + pos), axis.dot(vertices[0] + pos));
                
                for vertex in vertices.iter() {
                    let dot_product = axis.dot(*vertex + pos);

                    if dot_product < projection.min {
                        projection.min = dot_product;
                    } else if dot_product > projection.max {
                        projection.max = dot_product;
                    }
                }

                projection
            },
            Circle(r) => {
                // Since a circle has infinite vertices we calculate which one has the highest dot product
                // this will always be the direction of the axis and the negative direction of the axis
                let normalized = axis.normalized();
                let mut min = -normalized * *r;
                let mut max = normalized * *r;
                min += pos;
                max += pos;

                Projection {
                    min: axis.dot(min),
                    max: axis.dot(max),
                }
            },
        }
    }

    pub fn seperating_axis_test(t1: &Transform, c1: &CollisionShape, t2: &Transform, c2: &CollisionShape) -> (bool, Option<Vec2<f64>>) {                
        use CollisionShape::Circle;
        
        // Get separating axes
        let mut axes = vec![];

        // Circle on Circle check needs special case
        if let (Circle(r1), Circle(r2)) = (c1, c2) {
            // Check if circles are overlapping to avoid doing SAT if they aren't
            let x = (t1.x - t2.x).abs();
            let y = (t1.y - t2.y).abs();
            if x * x + y * y <= (r1 + r2) * (r1 + r2) {
                let axis = Vec2::new(t1.x - t2.x, t1.y - t2.y);
                axes.push(axis);
            } else {
                return (false, None);
            }
        } // Circle on Polygon check needs special case for separating axes
        else if c1.is_circle() {
            let axis = get_circle_polygon_axis(c1, t1, c2, t2);
            axes.push(axis);

            axes.append(&mut get_axes(&c2));
        }
        else if c2.is_circle() {
            let axis = get_circle_polygon_axis(c2, t2, c1, t1);
            axes.push(axis);

            axes.append(&mut get_axes(&c1));
        } else {
            axes.append(&mut get_axes(c1));
            axes.append(&mut get_axes(c2));

        }

        // Project shapes onto axes and check over overlapping
        let mut lowest: Option<f64> = None;
        let mut mtv: Option<Vec2<f64>> = None;

        for axis in axes.iter() {
            let p1 = project_shape(c1, t1, axis);
            let p2 = project_shape(c2, t2, axis);

            if !p1.overlaps(&p2) {
                return (false, None);
            } else {
                // Check if the overlapping area is the smallest we've found
                let overlap = p1.get_overlap(&p2);
                if lowest.is_none() || overlap < lowest.unwrap() {
                    lowest = Some(overlap);
                    mtv = Some(*axis);
                }
            }
        }

        // This code is only run if there was a collision and if there was a collision there will always be a lowest and an mtv
        let mtv = mtv.unwrap().normalized() * lowest.unwrap();
        (true, Some(mtv))
    }
}