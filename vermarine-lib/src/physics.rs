use shipyard::*;
use tetra::math::Vec2;

use crate::components::Transform;

pub fn physics_workload(world: &World) -> &'static str {
    let name = "Physics";
    
    world.add_unique(PhysicsWorld::new());
    let mut physics_bodies = world.borrow::<ViewMut<PhysicsBody>>();
    physics_bodies.update_pack();

    world.add_workload(name)
        .build();

    name
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

    // Lookup of EntityId to BodyId
    sparse: Vec<Option<(BodyId, EntityId)>>,
    // Lookup of transforms/colliders index to body_id
    reverse_sparse: Vec<Option<usize>>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            transforms: vec![],
            colliders: vec![],

            sparse: vec![],
            reverse_sparse: vec![],
        }
    }

    pub fn sync(&mut self, bodies: &mut ViewMut<PhysicsBody>) {
        // Adding bodies is done via add_body not with events

        // Remove bodies
        let removed = bodies.take_removed();
        let deleted = bodies.take_deleted().into_iter().map(|(id, _)| id);

        for id in removed.into_iter() {
            self.remove_body(id);
        }
        for id in deleted {
            self.remove_body(id);
        }
    }

    pub(crate) fn remove_body(&mut self, id: EntityId) {
        self.remove_overlapping(&PhysicsBody::new(id));

        let body = self.sparse[id.uindex()].clone().unwrap();
        let version = id.version();

        // Only remove body if the generation of the body matches the generation of the body that was asked to be removed
        if version != body.0.version {
            return;
        }

        // Check if we are removing the top body
        if body.0.index == self.transforms.len() - 1 {
            self.transforms.pop();
            self.colliders.pop();

            // Remove entry in sparse array
            self.sparse[id.uindex()] = None;
            self.reverse_sparse[body.0.index] = None;
        } else {
            // Replace removed_body with popped values to keep vec packed
            self.transforms[body.0.index] = self.transforms.pop().unwrap();
            self.colliders[body.0.index] = self.colliders.pop().unwrap();

            // Update sparse entry for popped body to point to body
            let popped_id = self.reverse_sparse[self.transforms.len()].unwrap();
            
            self.sparse[id.uindex()] = None;
            self.reverse_sparse[body.0.index] = Some(popped_id);
            self.reverse_sparse[self.transforms.len()] = None;
            self.sparse[popped_id].as_mut().unwrap().0.index = body.0.index;
        }
    }

    pub fn create_body(
        &mut self, 
        entities: &mut EntitiesViewMut, 
        bodies: &mut ViewMut<PhysicsBody>, 
        id: EntityId, 
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
                body.1 = id;
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
            
            self.sparse[sparse_index] = Some((body, id));
            self.transforms.push(transform);
            self.colliders.push(collider);
        }

        entities.add_component(bodies, PhysicsBody::new(id), id);
    }

    pub(crate) fn data_from_index_mut(&mut self, index: usize) -> (&mut Transform, &mut CollisionBody) {
        let transform = self.transforms.get_mut(index).unwrap();
        let collider = self.colliders.get_mut(index).unwrap();
        (transform, collider)
    }

    pub(crate) fn data_from_index(&self, index: usize) -> (&Transform, &CollisionBody) {
        let transform = self.transforms.get(index).unwrap();
        let collider = self.colliders.get(index).unwrap();
        (transform, collider)       
    }

    pub fn transform(&self, body: &PhysicsBody) -> &Transform {
        &self.transforms[self.sparse[body.id.uindex()].as_ref().unwrap().0.index]
    } 

    pub(crate) fn transform_mut(&mut self, body: &PhysicsBody) -> &mut Transform {
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

    pub(crate) fn parts_mut(&mut self, body: &PhysicsBody) -> (&mut Transform, &mut CollisionBody) {
        let index = self.index_from_body(body);
        (self.transforms.get_mut(index).unwrap(), self.colliders.get_mut(index).unwrap())
    }

    pub fn parts(&self, body: &PhysicsBody) -> (&Transform, &CollisionBody) {
        let index = self.index_from_body(body);
        (self.transforms.get(index).unwrap(), self.colliders.get(index).unwrap())
    }

    pub(crate) fn all_parts_mut(&mut self) -> (&mut [Transform], &mut [CollisionBody], &mut [Option<(BodyId, EntityId)>], &mut [Option<usize>]) {
        (&mut self.transforms, &mut self.colliders, &mut self.sparse, &mut self.reverse_sparse)
    }

    pub(crate) fn all_parts(&self) -> (&[Transform], &[CollisionBody], &[Option<(BodyId, EntityId)>], &[Option<usize>]) {
        (& self.transforms, & self.colliders, & self.sparse, & self.reverse_sparse)
    }

    //
    //

    /// Popping the returned vec of collisions will give you the most recent collision
    pub fn move_body_and_collide(&mut self, body: &PhysicsBody, delta: &Vec2<f64>) -> Vec<Collision> {
        let transform = self.transform_mut(body);
        transform.x += delta.x;
        transform.y += delta.y;

        self.handle_movement(body, true)
    }

    pub fn move_body(&mut self, body: &PhysicsBody, delta: &Vec2<f64>) {
        let transform = self.transform_mut(body);
        transform.x += delta.x;
        transform.y += delta.y;
        
        self.handle_movement(body, false);
    }

    pub fn move_body_to(&mut self, body: &PhysicsBody, position: &Vec2<f64>) {
        let transform = self.transform_mut(body);
        transform.x = position.x;
        transform.y = position.y;

        self.handle_movement(body, false);
    }

    pub fn move_body_to_x(&mut self, body: &PhysicsBody, x: f64) {
        let transform = self.transform_mut(body);
        transform.x = x;
    
        self.handle_movement(body, false);
    }

    pub fn move_body_to_y(&mut self, body: &PhysicsBody, y: f64) {
        let transform = self.transform_mut(body);
        transform.y = y;

        self.handle_movement(body, false);
    }

    //
    //

    pub(crate) fn handle_movement(&mut self, body: &PhysicsBody, resolve_collisions: bool) -> Vec<Collision> {
        self.remove_overlapping(body);
        self.update_overlapping(body, resolve_collisions)
    }

    /// Clears all stored overlapping data on the passed in body, also removes any overlapping data on other bodies regarding the passed in body
    pub(crate) fn remove_overlapping(&mut self, to_remove: &PhysicsBody) {
        for body in self.colliders.iter_mut() {
            body.remove_collision(to_remove.id);
        }

        let c_body = self.collider_mut(to_remove);
        c_body.remove_all_collisions();
    }

    /// Finds all overlapping bodies and adds collisions to them all
    pub(crate) fn update_overlapping(&mut self, body: &PhysicsBody, resolve_collisions: bool) -> Vec<Collision> {
        let index = self.sparse[body.id.uindex()].as_ref().unwrap().0.index;
        let (transforms, colliders, sparse, reverse_sparse) = self.all_parts_mut();
        let (tleft, t1, tright) = split_around_index_mut(transforms, index);
        let (cleft, c1, cright) = split_around_index_mut(colliders, index);

        let mut collisions = vec![];
        collisions.append(
            &mut Self::update_overlapping_partial(t1, c1, body.id, resolve_collisions, tleft, cleft, sparse, reverse_sparse, 0)
        );
        collisions.append(
            &mut Self::update_overlapping_partial(t1, c1, body.id, resolve_collisions, tright, cright, sparse, reverse_sparse, 1 + tleft.len())
        );
        collisions
    }

    /// Checks all colliders from c_body1 against all colliders from the provided slice
    pub(crate) fn update_overlapping_partial(t1: &mut Transform, c_body1: &mut CollisionBody, entity1: EntityId, resolve_collisions: bool, transforms: &mut [Transform], colliders: &mut [CollisionBody], sparse: &[Option<(BodyId, EntityId)>], reverse_sparse: &[Option<usize>], slice_offset: usize) -> Vec<Collision> {
        let mut collisions = vec![];
        let mut counter = slice_offset;
        for (t2, c_body2) in transforms.iter_mut().zip(colliders.iter_mut()) {
            // Calculate entities
            let entity2 = sparse[reverse_sparse[counter].unwrap()].as_ref().unwrap().1;
            counter += 1;

            // Sensor x Sensor
            for sensor1 in c_body1.sensors.iter_mut() {
                for sensor2 in c_body2.sensors.iter_mut() {
                    Self::update_overlapping_single(t1, sensor1, entity1, t2, sensor2, entity2, true, false);
                }
            }

            // Sensor1 x Collider2
            for sensor1 in c_body1.sensors.iter_mut() {
                for collider2 in c_body2.colliders.iter_mut() {
                    Self::update_overlapping_single(t1, sensor1, entity1, t2, collider2, entity2, false, false);
                }
            }

            // Sensor2 x Collider1
            for sensor2 in c_body2.sensors.iter_mut() {
                for collider1 in c_body1.colliders.iter_mut() {
                    Self::update_overlapping_single(t2, sensor2, entity2, t1, collider1, entity1, false, false);
                }
            }

            // Collider1 x Collider2
            for collider1 in c_body1.colliders.iter_mut() {
                for collider2 in c_body2.colliders.iter_mut() {
                    if let Some(collision) = Self::update_overlapping_single(t1, collider1, entity1, t2, collider2, entity2, true, resolve_collisions) {
                        collisions.push(collision);
                    }
                }
            }
        }
        collisions
    }

    pub(crate) fn update_overlapping_single(t1: &mut Transform, c1: &mut Collider, e1: EntityId, t2: &mut Transform, c2: &mut Collider, e2: EntityId, check_both: bool, resolve_collisions: bool) -> Option<Collision>{
        let (collided, mtv) = sat::seperating_axis_test(t1, &c1.shape, t2, &c2.shape);
        if collided {
            let collision = Self::handle_collision(t1, c1, t2, c2, e2, mtv, resolve_collisions);
            if check_both {
                Self::handle_collision(t2, c2, t1, c1, e1, mtv, false);
            }
            return collision;
        }
        None
    }

    pub(crate) fn handle_collision(t1: &mut Transform, c1: &mut Collider, t2: &Transform, c2: &Collider, e2: EntityId, mtv: Option<Vec2<f64>>, resolve_collisions: bool) -> Option<Collision> {
        if c1.collides_with & c2.collision_layer > 0 {
            let collision_data = Collision::new(t1.clone(), c1.shape.clone(), c1.collides_with, c1.collision_layer,
                t2.clone(), c2.shape.clone(), c2.collides_with, c2.collision_layer, e2, mtv.unwrap().normalized());
    
            c1.overlapping.push(collision_data.clone());

            if resolve_collisions {
                let mtv = mtv.unwrap();
                t1.x += mtv.x;
                t1.y += mtv.y;
            }

            return Some(collision_data);
        }
        None
    }
}

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

    pub(crate) fn transform_mut<'a>(&self, world: &'a mut PhysicsWorld) -> &'a mut Transform {
        world.transform_mut(self)
    }

    pub fn collider<'a>(&self, world: &'a PhysicsWorld) -> &'a CollisionBody {
        world.collider(self)
    }

    pub fn collider_mut<'a>(&self, world: &'a mut PhysicsWorld) -> &'a mut CollisionBody {
        world.collider_mut(self)
    }

    pub fn parts_mut<'a>(&self, world: &'a mut PhysicsWorld) -> (&'a Transform, &'a mut CollisionBody) {
        let (transform, collider) = world.parts_mut(self);
        (&*transform, collider)
    }

    pub fn parts<'a>(&self, world: &'a PhysicsWorld) -> (&'a Transform, &'a CollisionBody) {
        world.parts(self)
    }

    pub fn move_body_and_collide(&self, world: &mut PhysicsWorld, delta: &Vec2<f64>) -> Vec<Collision> {
        world.move_body_and_collide(self, delta)
    }

    pub fn move_body(&self, world: &mut PhysicsWorld, delta: &Vec2<f64>) {
        world.move_body(self, delta);
    }

    pub fn move_body_to(&self, world: &mut PhysicsWorld, position: &Vec2<f64>) {
        world.move_body_to(self, position);
    }

    pub fn move_body_to_x(&self, world: &mut PhysicsWorld, x: f64) {
        world.move_body_to_x(self, x);
    }

    pub fn move_body_to_y(&self, world: &mut PhysicsWorld, y: f64) {
        world.move_body_to_y(self, y);
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
}

impl CollisionBody {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_collider(collider: Collider) -> Self {
        CollisionBody {
            colliders: vec![collider],
            sensors: vec![],
        }
    }

    pub fn from_colliders(colliders: Vec<Collider>) -> Self {
        CollisionBody {
            colliders,
            sensors: vec![],
        }
    }

    pub fn from_sensor(sensor: Collider) -> Self {
        CollisionBody {
            colliders: vec![],
            sensors: vec![sensor],
        }
    }

    pub fn from_sensors(sensors: Vec<Collider>) -> Self {
        CollisionBody {
            colliders: vec![],
            sensors,
        }
    }

    pub fn from_parts(colliders: Vec<Collider>, sensors: Vec<Collider>) -> Self {
        CollisionBody {
            colliders,
            sensors,
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
        if Vec2::new(t2.x - t1.x, t2.y - t1.y).dot(mtv) > 0.0 {
            mtv *= -1.0;
        }
        
        (true, Some(mtv))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bug_reproduction() {
        use crate::*;
        use shipyard::*;

        let world = World::new();
        super::physics_workload(&world);

        // Setup
        let [e1, e2] = world.run(|
            mut entities: EntitiesViewMut,
            mut bodies: ViewMut<PhysicsBody>,
            mut physics_world: UniqueViewMut<PhysicsWorld>| { 
                e1 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    &e1.unwrap(), 
                    Transform::new(10.0, 10.0), 
                    CollisionBody::from_collider(Collider::half_extents(2.0, 2.0, 1, 2)),
                );

                e2 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    &e2.unwrap(), 
                    Transform::new(0.0, 0.0), 
                    CollisionBody::from_collider(Collider::half_extents(2.0, 2.0, 2, 1)),
                );
                
                // Move e2 into e1
                let body = bodies.get(e2.unwrap()).unwrap();
                body.move_body(&mut physics_world, &tetra::math::Vec2::new(10.0, 10.0));

                assert_eq!(body.collider(&mut physics_world).colliders[0].overlapping.len(), 1);
                assert_eq!(body.collider(&mut physics_world).colliders[0].overlapping[0].entity2, e1.unwrap());

                let body = bodies.get(e1.unwrap()).unwrap();
                assert_eq!(body.collider(&mut physics_world).colliders[0].overlapping.len(), 1);
                assert_eq!(body.collider(&mut physics_world).colliders[0].overlapping[0].entity2, e2.unwrap());

                [e1, e2]
        });

        // Kill entity
        world.run(|mut all_storages: AllStoragesViewMut| {
            let mut to_kill = None;
            {
                let (mut bodies, mut physics_world) = all_storages.borrow::<(ViewMut<PhysicsBody>, UniqueViewMut<PhysicsWorld>)>();

                // Check for collision
                let body = bodies.get(e1).unwrap();
                let collision_body = body.collider(&mut physics_world);
                
                assert_eq!(collision_body.colliders[0].overlapping.len(), 1);

                for collision in collision_body.colliders[0].overlapping.iter() {
                    to_kill = Some(collision.entity2);
                }
            }

            all_storages.delete(to_kill.unwrap());
            let (mut world, mut bodies) = all_storages.borrow::<(UniqueViewMut<PhysicsWorld>, ViewMut<PhysicsBody>)>();
            world.sync(&mut bodies);

            let body = bodies.get(e1).unwrap();
            let collision_body = body.collider(&mut world);
            assert_eq!(collision_body.colliders[0].overlapping.len(), 0);
        });

        // Move alive entity
        world.run(|
            mut bodies: ViewMut<PhysicsBody>,
            mut world: UniqueViewMut<PhysicsWorld>,| {
                let body = bodies.get(e1);
                body.move_body(&mut world, &tetra::math::Vec2::new(-10.0, -10.0));
                assert_eq!(body.collider(&mut world).colliders[0].overlapping.len(), 0);
        });
    }

    #[test]
    fn bug_two() {
        use crate::*;
        use shipyard::*;

        let world = World::new();
        super::physics_workload(&world);

        // Setup
        let [e1, e2] = world.run(|
            mut entities: EntitiesViewMut,
            mut bodies: ViewMut<PhysicsBody>,
            mut physics_world: UniqueViewMut<PhysicsWorld>| { 
                e1 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    &e1.unwrap(), 
                    Transform::new(10.0, 10.0), 
                    CollisionBody::from_sensor(Collider::half_extents(2.0, 2.0, 1, 2)),
                );

                (0..100).for_each(|_| { entities.add_entity((), ()); });

                e2 = entities.add_entity((), ());
                physics_world.create_body(
                    &mut entities, 
                    &mut bodies, 
                    &e2.unwrap(), 
                    Transform::new(0.0, 0.0), 
                    CollisionBody::from_collider(Collider::half_extents(2.0, 2.0, 2, 1)),
                );
                
                println!("E1: {:?}\nE2: {:?}\n", e1, e2);

                // Move e2 into e1
                let body = bodies.get(e2).unwrap();
                body.move_body(&mut physics_world, &tetra::math::Vec2::new(10.0, 10.0));

                assert_eq!(body.collider(&mut physics_world).colliders[0].overlapping.len(), 0);

                let body = bodies.get(e1).unwrap();
                assert_eq!(body.collider(&mut physics_world).sensors[0].overlapping.len(), 1);
                assert_eq!(body.collider(&mut physics_world).sensors[0].overlapping[0].entity2, e2);
        });

        // Kill entity
        world.run(|mut all_storages: AllStoragesViewMut| {
            let mut to_kill = None;
            {
                let (mut bodies, mut physics_world) = all_storages.borrow::<(ViewMut<PhysicsBody>, UniqueViewMut<PhysicsWorld>)>();

                // Check for collision
                let body = bodies.get(e1).unwrap();
                let collision_body = body.collider(&mut physics_world);
                
                assert_eq!(collision_body.sensors[0].overlapping.len(), 1);

                for collision in collision_body.sensors[0].overlapping.iter() {
                    to_kill = Some(collision.entity2);
                }
            }

            all_storages.delete(to_kill.unwrap());
            let (mut world, mut bodies) = all_storages.borrow::<(UniqueViewMut<PhysicsWorld>, ViewMut<PhysicsBody>)>();
            world.sync(&mut bodies);

            let body = bodies.get(e1).unwrap();
            let collision_body = body.collider(&mut world);
            assert_eq!(collision_body.sensors[0].overlapping.len(), 0);
        });

        // Move alive entity
        world.run(|
            mut bodies: ViewMut<PhysicsBody>,
            mut world: UniqueViewMut<PhysicsWorld>,| {
                let body = bodies.get(e1).unwrap();
                body.move_body(&mut world, &tetra::math::Vec2::new(-10.0, -10.0));
                assert_eq!(body.collider(&mut world).sensors[0].overlapping.len(), 0);
        });
    }
}