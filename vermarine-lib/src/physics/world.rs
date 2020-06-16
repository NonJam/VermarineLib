use super::*;

pub struct PhysicsWorld {
    // Body data
    transforms: Vec<Transform>,
    colliders: Vec<CollisionBody>,
    owners: Vec<EntityId>,

    // Lookup of EntityId to BodyId
    sparse: Vec<Option<usize>>,

    broadphase: SpatialBuckets,
}

impl PhysicsWorld {
    pub fn new(bucket_width: f64, bucket_height: f64) -> Self {
        PhysicsWorld {
            transforms: vec![],
            colliders: vec![],
            owners: vec![],

            sparse: vec![],

            broadphase: SpatialBuckets::new(bucket_height, bucket_width),
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
        self.remove_overlapping(id);

        {
            let transform = &self.transform(id).clone();
            let aabb = &self.collider(id).aabb.clone();
            self.broadphase.remove(id, transform, aabb);
        }

        let body = self.sparse[id.uindex()].clone().unwrap();

        // Only remove body if the generation of the body matches the generation of the body that was asked to be removed
        if id != self.owners[body] {
            return;
        }

        // Check if we are removing the top body
        if body == self.transforms.len() - 1 {
            self.transforms.pop();
            self.colliders.pop();
            self.owners.pop();

            // Remove entry in sparse array
            self.sparse[id.uindex()] = None;
        } else {
            // Replace removed_body with popped values to keep vec packed
            self.transforms[body] = self.transforms.pop().unwrap();
            self.colliders[body] = self.colliders.pop().unwrap();
            self.owners[body] = self.owners.pop().unwrap();

            self.sparse[id.uindex()] = None;
            let owner = self.owners[body].uindex();
            self.sparse[owner] = Some(body);
        }
    }

    pub fn create_body(
        &mut self, 
        entities: &mut EntitiesViewMut, 
        bodies: &mut ViewMut<PhysicsBody>, 
        id: EntityId, 
        transforms: &mut ViewMut<Transform>,
        transform: Transform, 
        collider: CollisionBody
    ) {
        let sparse_index = id.uindex();

        {
            let transform = &transform.clone();
            let aabb = &collider.aabb.clone();
            self.broadphase.insert(id, transform, aabb);
        }

        // Padding 
        if sparse_index >= self.sparse.len() {
            // Pad sparse vec so that we can directly index sparse with entity id
            let padding = (sparse_index - self.sparse.len()) + 1;
            (0..padding).for_each(|_| self.sparse.push(None));
        } 

        // Dont replace body if it's a higher generation than the passed in id
        if let Some(body) = &mut self.sparse[sparse_index] {
            let body = *body;
            if self.owners[body].gen() > id.gen() {
                return;
            } else {
                // Replace current body with passed in body
                self.owners[body] = id;
                self.transforms[body] = transform;
                self.colliders[body] = collider;
                return;
            }
        } else {
            // Create new body
            let body = self.transforms.len();            
            self.sparse[sparse_index] = Some(body);

            self.owners.push(id);
            self.transforms.push(transform);
            self.colliders.push(collider);
        }

        entities.add_component(bodies, PhysicsBody, id);
        entities.add_component(transforms, transform, id);
    }

    //
    //

    /// Popping the returned vec of collisions will give you the most recent collision
    pub fn move_body_and_collide(&mut self, body: EntityId, delta: Vec2<f64>) -> Vec<Collision> {
        self.handle_pre_movement(body);

        let transform = self.transform_mut(body);
        transform.x += delta.x;
        transform.y += delta.y;

        self.handle_movement(body, true)
    }

    pub fn move_body(&mut self, body: EntityId, delta: Vec2<f64>) {
        self.handle_pre_movement(body);

        let transform = self.transform_mut(body);
        transform.x += delta.x;
        transform.y += delta.y;
        
        self.handle_movement(body, false);
    }

    pub fn move_body_to(&mut self, body: EntityId, position: Vec2<f64>) {
        self.handle_pre_movement(body);

        let transform = self.transform_mut(body);
        transform.x = position.x;
        transform.y = position.y;

        self.handle_movement(body, false);
    }

    pub fn move_body_to_x(&mut self, body: EntityId, x: f64) {
        self.handle_pre_movement(body);

        let transform = self.transform_mut(body);
        transform.x = x;
    
        self.handle_movement(body, false);
    }

    pub fn move_body_to_y(&mut self, body: EntityId, y: f64) {
        self.handle_pre_movement(body);
        
        let transform = self.transform_mut(body);
        transform.y = y;

        self.handle_movement(body, false);
    }

    //
    //

    pub(crate) fn handle_pre_movement(&mut self, id: EntityId) {
        self.remove_overlapping(id);

        {
            let transform = &self.transform(id).clone();
            let aabb = &self.collider(id).aabb.clone();
            self.broadphase.remove(id, transform, aabb);
        }
    }

    pub(crate) fn handle_movement(&mut self, id: EntityId, resolve_collisions: bool) -> Vec<Collision> {
        let collisions = self.update_overlapping(id, resolve_collisions);

        {
            let transform = &self.transform(id).clone();
            let aabb = &self.collider(id).aabb.clone();
            self.broadphase.insert(id, transform, aabb);
        }

        collisions
    }

    /// Clears all stored overlapping data on the passed in body, also removes any overlapping data on other bodies regarding the passed in body
    pub(crate) fn remove_overlapping(&mut self, to_remove: EntityId) {

        let transform = &self.transform(to_remove).clone();
        let aabb = &self.collider(to_remove).aabb.clone();
        for id in self.broadphase.nearby(to_remove, transform, aabb).into_iter() {
            let body = self.collider_mut(id);
            body.remove_collision(to_remove);
        }

        let c_body = self.collider_mut(to_remove);
        c_body.remove_all_collisions();
    }

    /// Finds all overlapping bodies and adds collisions to them all
    pub(crate) fn update_overlapping(&mut self, body: EntityId, resolve_collisions: bool) -> Vec<Collision> {
        let mut collisions = vec![];
        let transform = &self.transform(body).clone();
        let aabb = &self.collider(body).aabb.clone();
        let nearby = self.broadphase.nearby(body, transform, aabb);
        for id in nearby.into_iter() {
            let body1 = self.sparse[body.uindex()].unwrap();
            let body2 = self.sparse[id.uindex()].unwrap();

            assert_ne!(body1, body2);

            let (transforms, colliders, _, _) = self.all_parts_mut();
            let (t1, c1, t2, c2) = if body1 > body2 {
                let (tleft, tright) = transforms.split_at_mut(body1);
                let (cleft, cright) = colliders.split_at_mut(body1);

                (
                    &mut tright[0],
                    &mut cright[0],
                    &mut tleft[body2],
                    &mut cleft[body2],
                )
            } else {
                let (tleft, tright) = transforms.split_at_mut(body2);
                let (cleft, cright) = colliders.split_at_mut(body2);

                (
                    &mut tleft[body1],
                    &mut cleft[body1],
                    &mut tright[0],
                    &mut cright[0],
                )
            };

            collisions.append(
                &mut Self::update_overlapping_partial(t1, c1, body, t2, c2, id, resolve_collisions)
            );
        }
        collisions
    }

    /// Checks all colliders from c_body1 against all colliders from the provided slice
    pub(crate) fn update_overlapping_partial(t1: &mut Transform, c_body1: &mut CollisionBody, entity1: EntityId, t2: &mut Transform, c_body2: &mut CollisionBody, entity2: EntityId, resolve_collisions: bool) -> Vec<Collision> {
        let mut collisions = vec![];
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
        collisions
    }

    pub(crate) fn update_overlapping_single(t1: &mut Transform, c1: &mut Collider, e1: EntityId, t2: &mut Transform, c2: &mut Collider, e2: EntityId, check_both: bool, resolve_collisions: bool) -> Option<Collision>{
        let mut result: Option<(bool, Option<Vec2<f64>>)> = None;
        let mut collision = None;

        if c1.collides_with & c2.collision_layer > 0 {
            result = Some(sat::seperating_axis_test(t1, &c1.shape, t2, &c2.shape));
            let (collided, mtv) = result.unwrap();
            if collided {
                collision = Some(
                    Self::handle_collision(t1, c1, t2, c2, e2, mtv, resolve_collisions)
                )
            }
        }

        if c2.collides_with & c1.collision_layer > 0 && check_both {
            if result.is_none() {
                result = Some(sat::seperating_axis_test(t1, &c1.shape, t2, &c2.shape));
            }
            let (collided, mtv) = result.unwrap();
            
            if collided {
                Self::handle_collision(t2, c2, t1, c1, e1, Some(-mtv.unwrap()), false);
            }
        }
        collision
    }

    pub(crate) fn handle_collision(t1: &mut Transform, c1: &mut Collider, t2: &Transform, c2: &Collider, e2: EntityId, mtv: Option<Vec2<f64>>, resolve_collisions: bool) -> Collision {
        let collision_data = Collision::new(t1.clone(), c1.shape.clone(), c1.collides_with, c1.collision_layer,
            t2.clone(), c2.shape.clone(), c2.collides_with, c2.collision_layer, e2, mtv.unwrap().normalized());

        c1.overlapping.push(collision_data.clone());

        if resolve_collisions {
            let mtv = mtv.unwrap();
            t1.x += mtv.x;
            t1.y += mtv.y;
        }

        collision_data
    }

    //
    //

    #[allow(dead_code)]
    pub(crate) fn data_from_index_mut(&mut self, index: usize) -> (&mut Transform, &mut CollisionBody) {
        let transform = self.transforms.get_mut(index).unwrap();
        let collider = self.colliders.get_mut(index).unwrap();
        (transform, collider)
    }
    #[allow(dead_code)]
    pub(crate) fn data_from_index(&self, index: usize) -> (&Transform, &CollisionBody) {
        let transform = self.transforms.get(index).unwrap();
        let collider = self.colliders.get(index).unwrap();
        (transform, collider)       
    }
    pub fn transform(&self, body: EntityId) -> &Transform {
        &self.transforms[self.sparse[body.uindex()].unwrap()]
    } 
    pub(crate) fn transform_mut(&mut self, body: EntityId) -> &mut Transform {
        &mut self.transforms[self.sparse[body.uindex()].unwrap()]
    } 
    pub fn collider(&self, body: EntityId) -> &CollisionBody {
        &self.colliders[self.sparse[body.uindex()].unwrap()]
    } 
    pub fn collider_mut(&mut self, body: EntityId) -> &mut CollisionBody {
        &mut self.colliders[self.sparse[body.uindex()].unwrap()]
    } 
    pub fn index_from_body(&self, body: EntityId) -> usize {
        self.sparse[body.uindex()].unwrap()
    }
    pub fn parts_mut(&mut self, body: EntityId) -> (&Transform, &mut CollisionBody) {
        let index = self.index_from_body(body);
        (self.transforms.get(index).unwrap(), self.colliders.get_mut(index).unwrap())
    } 
    #[allow(dead_code)]
    pub(crate) fn parts_mut_real(&mut self, body: EntityId) -> (&mut Transform, &mut CollisionBody) {
        let index = self.index_from_body(body);
        (self.transforms.get_mut(index).unwrap(), self.colliders.get_mut(index).unwrap())
    }
    pub fn parts(&self, body: EntityId) -> (&Transform, &CollisionBody) {
        let index = self.index_from_body(body);
        (self.transforms.get(index).unwrap(), self.colliders.get(index).unwrap())
    }
    pub(crate) fn all_parts_mut(&mut self) -> (&mut [Transform], &mut [CollisionBody], &mut [EntityId], &mut [Option<usize>]) {
        (&mut self.transforms, &mut self.colliders, &mut self.owners, &mut self.sparse)
    }
    #[allow(dead_code)]
    pub(crate) fn all_parts(&self) -> (&[Transform], &[CollisionBody], &[EntityId], &[Option<usize>]) {
        (&self.transforms, &self.colliders, &self.owners, &self.sparse,)
    }
}