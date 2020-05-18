use shipyard::*;
use tetra::math::Vec2;

use crate::components::Transform;

pub fn physics_workload(world: &World) -> &'static str {
    let name = "Physics";
    
    world.add_unique(PhysicsWorld::new());
    let mut physics_bodies = world.borrow::<ViewMut<PhysicsBody>>();
    physics_bodies.update_pack();

    world.add_workload(name)
        .with_system(system!(clear_collisions))
        .with_system(system!(calc_collisions))
        .build();

    name
}

pub fn clear_collisions(mut physics_bodies: ViewMut<PhysicsBody>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    for physic_body in (&mut physics_bodies).iter() {
        let body = physic_body.collider_mut(&mut physics_world);
        for collider in body.colliders.iter_mut() {
            collider.overlapping.clear();
        }
    }
}

pub fn calc_collisions(mut physics_bodies: ViewMut<PhysicsBody>, mut physics_world: UniqueViewMut<PhysicsWorld>) {
    let len = physics_world.transforms.len();
    for index in 0..len {
        let (transforms, colliders, sparse, reverse_sparse) = physics_world.all_parts_mut();

        let transforms = &mut transforms.split_at_mut(index);
        let colliders = &mut colliders.split_at_mut(index);

        let (t1, c1) = (&mut transforms.1[0], &mut colliders.1[0]);
    
        let mut counter = 0;
        for (t2, c2) in transforms.0.iter_mut().zip(colliders.0.iter_mut()) {
            let entity2 = sparse[reverse_sparse[counter].unwrap()].as_ref().unwrap().1;

            let c1 = &mut c1.colliders[0];
            let c2 = &mut c2.colliders[0];
            let (collided, _) = sat::seperating_axis_test(&t1, &c1.shape, t2, &c2.shape);
            if collided && c1.collides_with & c2.collision_layer > 0 {
                let collision = Collision::new(t1.clone(), c1.shape.clone(), c1.collides_with, c1.collision_layer,
                    t2.clone(), c2.shape.clone(), c2.collides_with, c2.collision_layer, entity2);
                c1.overlapping.push(collision.clone()); 
            }

            counter += 1;
        }
    
        if let (Some((t1, tright)), Some((c1, cright))) = (transforms.1.split_first_mut(), colliders.1.split_first_mut()) {

            let mut counter = index + 1;
            for (t2, c2) in tright.iter_mut().zip(cright.iter_mut()) {
                let entity2 = sparse[reverse_sparse[counter].unwrap()].as_ref().unwrap().1;

                let c1 = &mut c1.colliders[0];
                let c2 = &mut c2.colliders[0];
                let (collided, _) = sat::seperating_axis_test(&t1, &c1.shape, t2, &c2.shape);
                if collided && c1.collides_with & c2.collision_layer > 0 {
                    let collision = Collision::new(t1.clone(), c1.shape.clone(), c1.collides_with, c1.collision_layer,
                        t2.clone(), c2.shape.clone(), c2.collides_with, c2.collision_layer, entity2);
                    c1.overlapping.push(collision.clone()); 
                }

                counter += 1;
            }
        }
    }
}

#[derive(Clone)]
pub struct BodyId {
    index: usize,
    version: u64,
}

impl BodyId {
    pub fn new(index: usize, version: u64) -> Self {
        BodyId {
            index,
            version,
        }
    }
}

pub struct PhysicsWorld {
    transforms: Vec<Transform>,
    colliders: Vec<CollisionBody>,

    // Lookup of body_id to transforms/colliders index
    // first is index, second is generation
    sparse: Vec<Option<(BodyId, EntityId)>>,
    // Lookup of transforms/colliders index to body_id
    reverse_sparse: Vec<Option<usize>>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            transforms: vec![],
            colliders: vec![],

            sparse: vec![None],
            reverse_sparse: vec![],
        }
    }

    pub fn sync(&mut self, bodies: &mut ViewMut<PhysicsBody>) {
        // Adding bodies is done via add_body not with events

        // Remove bodies
        let removed = bodies.take_removed();
        let deleted = bodies.take_deleted().into_iter().map(|(id, _)| id).collect::<Vec<EntityId>>();

        for id in removed.iter() {
            self.remove_body(id);
        }
        for id in deleted.iter() {
            self.remove_body(id);
        }
    }

    pub(crate) fn remove_body(&mut self, id: &EntityId) {
        let body = self.sparse[id.uindex()].clone().unwrap();
        let version = id.version();

        // Only remove body if the generation of the body matches the generation of the body that was asked to be removed
        if version != self.sparse[id.uindex()].as_ref().unwrap().0.version {
            return;
        }

        // Remove entry in sparse array
        self.sparse[id.uindex()] = None;
        self.reverse_sparse[body.0.index] = None;

        // Check if we are removing the top body
        if body.0.index == self.transforms.len() - 1 {
            self.transforms.pop();
            self.colliders.pop();
        } else {
            // Replace removed_body with popped values to keep vec packed
            self.transforms[body.0.index] = self.transforms.pop().unwrap();
            self.colliders[body.0.index] = self.colliders.pop().unwrap();

            // Update sparse entry for popped body to point to body
            let popped_id = self.reverse_sparse[self.transforms.len()].unwrap();
            self.sparse[popped_id].as_mut().unwrap().0.index = body.0.index;
            self.sparse[popped_id].as_mut().unwrap().1 = *id;

            // Update reverse sparse
            self.reverse_sparse[self.transforms.len()] = None;
            self.reverse_sparse[body.0.index] = Some(popped_id);
        }
    }

    pub fn create_body(
        &mut self, 
        entities: &mut EntitiesViewMut, 
        bodies: &mut ViewMut<PhysicsBody>, 
        id: &EntityId, 
        transform: Transform, 
        collider: CollisionBody
    ) {
        let sparse_index = id.uindex();

        // Padding 
        if sparse_index >= self.sparse.len() {
            // Pad sparse vec so that we can directly index sparse with entity id
            let padding = (sparse_index - self.sparse.len()) + 1;
            (0..padding).for_each(|_| self.sparse.push(None));
        } 

        // Dont replace body if it's a higher version than the passed in id
        if let Some(body) = &mut self.sparse[sparse_index] {
            if body.0.version > id.version() {
                return;
            } else {
                // Replace current body with passed in body
                body.0.version = id.version();
                body.1 = *id;
                self.transforms[body.0.index] = transform;
                self.colliders[body.0.index] = collider;
                return;
            }
        } else {
            // Create new body
            let body = BodyId::new(self.transforms.len(), id.version());
            
            // set reverse sparse
            if body.index >= self.reverse_sparse.len() {
                let padding = (self.reverse_sparse.len() - body.index) + 1;
                (0..padding).for_each(|_| self.reverse_sparse.push(None));
            }
            self.reverse_sparse[body.index] = Some(sparse_index);
            
            self.sparse[sparse_index] = Some((body, *id));
            self.transforms.push(transform);
            self.colliders.push(collider);
        }

        entities.add_component(bodies, PhysicsBody::new(*id), *id);
    }

    pub fn data_from_index(&mut self, index: usize) -> (&mut Transform, &mut CollisionBody) {
        let transform = self.transforms.get_mut(index).unwrap();
        let collider = self.colliders.get_mut(index).unwrap();
        (transform, collider)
    }

    pub fn transform(&self, body: &PhysicsBody) -> &Transform {
        &self.transforms[self.sparse[body.id.uindex()].as_ref().unwrap().0.index]
    } 

    pub fn transform_mut(&mut self, body: &PhysicsBody) -> &mut Transform {
        &mut self.transforms[self.sparse[body.id.uindex()].as_ref().unwrap().0.index]
    } 

    pub fn collider(&self, body: &PhysicsBody) -> &CollisionBody {
        &self.colliders[self.sparse[body.id.uindex()].as_ref().unwrap().0.index]
    } 

    pub fn collider_mut(&mut self, body: &PhysicsBody) -> &mut CollisionBody {
        &mut self.colliders[self.sparse[body.id.uindex()].as_ref().unwrap().0.index]
    } 

    pub fn index_from_body(&self, body: &PhysicsBody) -> usize {
        self.sparse[body.id.uindex()].as_ref().unwrap().0.index
    }

    pub fn parts_mut(&mut self, body: &PhysicsBody) -> (&mut Transform, &mut CollisionBody) {
        let index = self.index_from_body(body);
        (self.transforms.get_mut(index).unwrap(), self.colliders.get_mut(index).unwrap())
    }

    pub fn all_parts_mut(&mut self) -> (&mut [Transform], &mut [CollisionBody], &mut [Option<(BodyId, EntityId)>], &mut [Option<usize>]) {
        (&mut self.transforms[..], &mut self.colliders[..], &mut self.sparse, &mut self.reverse_sparse)
    }

    //
    //

    pub fn move_body_and_collide(&mut self, body: &PhysicsBody, delta: &Vec2<f64>) {
        let body = self.sparse[body.id.uindex()].as_ref().unwrap();

        let transforms = &mut self.transforms.split_at_mut(body.0.index);
        let colliders = &mut self.colliders.split_at_mut(body.0.index);
        let (t1, c1) = (&mut transforms.1[0], &mut colliders.1[0]);
        t1.x += delta.x;
        t1.y += delta.y;

        for (transform, collider) in transforms.0.iter_mut().zip(colliders.0.iter_mut()) {
            let (collided, mtv) = sat::seperating_axis_test(&t1, &c1.colliders[0].shape, transform, &collider.colliders[0].shape);
            if collided && c1.colliders[0].collides_with & collider.colliders[0].collision_layer > 0 {
                t1.x -= mtv.unwrap().x;
                t1.y -= mtv.unwrap().y;
            }
        }

        if let (Some((tleft, tright)), Some((cleft, cright))) = (transforms.1.split_first_mut(), colliders.1.split_first_mut()) {
            for (transform, collider) in tright.iter_mut().zip(cright.iter_mut()) {
                let (collided, mtv) = sat::seperating_axis_test(&tleft, &cleft.colliders[0].shape, transform, &collider.colliders[0].shape);
                if collided && cleft.colliders[0].collides_with & collider.colliders[0].collision_layer > 0 {
                    tleft.x -= mtv.unwrap().x;
                    tleft.y -= mtv.unwrap().y;
                }
            }
        }
    }

    pub fn move_body(&mut self, body: &PhysicsBody, delta: &Vec2<f64>) {
        let transform = self.transform_mut(body);
        transform.x += delta.x;
        transform.y += delta.y;
    }
}

pub struct PhysicsBody {
    id: EntityId,
}

impl PhysicsBody {
    pub(crate) fn new(id: EntityId) -> Self {
        PhysicsBody {
            id
        }
    }

    pub fn transform<'a>(&self, world: &'a PhysicsWorld) -> &'a Transform {
        world.transform(self)
    }

    pub fn transform_mut<'a>(&self, world: &'a mut PhysicsWorld) -> &'a mut Transform {
        world.transform_mut(self)
    }

    pub fn collider<'a>(&self, world: &'a PhysicsWorld) -> &'a CollisionBody {
        world.collider(self)
    }

    pub fn collider_mut<'a>(&self, world: &'a mut PhysicsWorld) -> &'a mut CollisionBody {
        world.collider_mut(self)
    }

    pub fn parts_mut<'a>(&self, world: &'a mut PhysicsWorld) -> (&'a mut Transform, &'a mut CollisionBody) {
        world.parts_mut(self)
    }

    pub fn move_body_and_collide(&self, world: &mut PhysicsWorld, delta: &Vec2<f64>) {
        world.move_body_and_collide(self, delta);
    }

    pub fn move_body(&self, world: &mut PhysicsWorld, delta: &Vec2<f64>) {
        world.move_body(self, delta);
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
                    axes1.push(normal.normalized());
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

            return axis.normalized();
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
                axes.push(axis.normalized());
            } else {
                return (false, None);
            }
        } // Circle on Polygon check needs special case for separating axes
        else if c1.is_circle() {
            let axis = get_circle_polygon_axis(c1, t1, c2, t2);
            axes.push(axis.normalized());

            axes.append(&mut get_axes(&c2));
        }
        else if c2.is_circle() {
            let axis = get_circle_polygon_axis(c2, t2, c1, t1);
            axes.push(axis.normalized());

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
                if lowest.is_none() || overlap <= lowest.unwrap() {
                    lowest = Some(overlap);
                    mtv = Some(*axis);
                }
            }
        }

        // This code is only run if there was a collision and if there was a collision there will always be a lowest and an mtv
        let mut mtv = mtv.unwrap() * lowest.unwrap();
        if Vec2::new(t2.x - t1.x, t2.y - t1.y).dot(mtv) < 0.0 {
            mtv *= -1.0;
        }
        
        (true, Some(mtv))
    }
}