use super::*;

pub struct SpatialBuckets {
    buckets: Vec<Vec<EntityId>>,
    bucket_width: f64,
    bucket_height: f64,
    width: usize,
    height: usize,
}

impl SpatialBuckets {
    pub fn new(bucket_width: f64, bucket_height: f64) -> Self {
        SpatialBuckets {
            buckets: vec![vec![]],
            bucket_width,
            bucket_height,
            width: 1,
            height: 1,
        }
    }

    pub fn insert(&mut self, id: EntityId, transform: &Transform, aabb: &AABB) {
        let xmin = transform.x + aabb.dx;
        let ymin = transform.y + aabb.dy;
        let xmax = xmin + aabb.width;
        let ymax = ymin + aabb.height;

        let (xmin, ymin) = self.point_to_cell(xmin, ymin);
        let (xmax, ymax) = self.point_to_cell(xmax, ymax);

        while 
            self.wrap_point(xmin) >= self.width || 
            self.wrap_point(xmax) >= self.width || 
            self.wrap_point(ymin) >= self.height || 
            self.wrap_point(ymax) >= self.height {
            self.resize();
        }

        for x in xmin..=xmax {
            for y in ymin..=ymax {
                let (x, y) = self.wrap_cell(x, y);
                self.buckets[y * self.width + x].push(id);
            }
        }
    }

    pub fn remove(&mut self, id: EntityId, transform: &Transform, aabb: &AABB) {
        let xmin = transform.x + aabb.dx;
        let ymin = transform.y + aabb.dy;
        let xmax = xmin + aabb.width;
        let ymax = ymin + aabb.height;

        let (xmin, ymin) = self.point_to_cell(xmin, ymin);
        let (xmax, ymax) = self.point_to_cell(xmax, ymax);

        while 
            self.wrap_point(xmin) >= self.width || 
            self.wrap_point(xmax) >= self.width || 
            self.wrap_point(ymin) >= self.height || 
            self.wrap_point(ymax) >= self.height {
            self.resize();
        }

        for x in xmin..=xmax {
            for y in ymin..=ymax {
                let (x, y) = self.wrap_cell(x, y);
                self.buckets[y * self.width + x].retain(|&v| v != id );
            }
        }
    }

    pub fn nearby(&mut self, id: EntityId, transform: &Transform, aabb: &AABB) -> Vec<EntityId> {
        let xmin = transform.x + aabb.dx;
        let ymin = transform.y + aabb.dy;
        let xmax = xmin + aabb.width;
        let ymax = ymin + aabb.height;

        let (xmin, ymin) = self.point_to_cell(xmin, ymin);
        let (xmax, ymax) = self.point_to_cell(xmax, ymax);

        while 
            self.wrap_point(xmin) >= self.width || 
            self.wrap_point(xmax) >= self.width || 
            self.wrap_point(ymin) >= self.height || 
            self.wrap_point(ymax) >= self.height {
            self.resize();
        }

        let mut nearby = vec![];
        for x in xmin..=xmax {
            for y in ymin..=ymax {
                let (x, y) = self.wrap_cell(x, y);
                for e in self.buckets[y * self.width + x].iter() {
                    if *e != id && !nearby.contains(e) {
                        nearby.push(*e);
                    }
                }
            }
        }
        nearby
    }

    pub fn resize(&mut self) {
        let mut insert_idx = self.width;
        for _ in 0..self.height {
            (0..self.width).for_each(|_| self.buckets.insert(insert_idx, vec![]));
            insert_idx += self.width * 2;
        }
        self.buckets.append(&mut vec![vec![]; self.width * 2 * self.height]);
        self.width *= 2;
        self.height *= 2;
    }

    pub fn point_to_cell(&self, x: f64, y: f64) -> (isize, isize) {
        let x = f64::floor(x / self.bucket_width) as isize;
        let y = f64::floor(y / self.bucket_height) as isize;

        (x, y)
    }

    pub fn wrap_point(&self, point: isize) -> usize {
        let mut point = point * 2;
        if point < 0 { 
            point *= -1;
            point -= 1;
        }
        point as usize
    }

    pub fn wrap_cell(&self, x: isize, y: isize) -> (usize, usize) {
        (self.wrap_point(x), self.wrap_point(y))
    }
}

//
//

#[cfg(test)]
mod tests {
    #[test]
    fn insert() {
        use crate::physics::*;

        let world = World::new();
        let id = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb1 = AABB::new(0.0, 0.0, 3.0, 3.0);
        buckets.insert(id, &Transform::new(5.0, 5.0), &aabb1);

        assert_eq!(buckets.buckets[0][0], id);
        assert_eq!(buckets.buckets.len(), 1);
    }

    #[test]
    fn resize() {
        use crate::physics::*;

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        buckets.resize();

        assert_eq!(buckets.width, 2);
        assert_eq!(buckets.height, 2);
        assert_eq!(buckets.buckets.len(), 4);
    }

    #[test]
    fn cross_boundary_insert() {
        use crate::physics::*;

        let world = World::new();
        let id = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb = AABB::new(0.0, 0.0, 10.0, 10.0);
        buckets.insert(id, &Transform::new(5.0, 5.0), &aabb);

        for bucket in buckets.buckets.iter() {
            println!("{}", bucket.len());
        }
        assert_eq!(buckets.buckets[0][0], id);
        assert_eq!(buckets.buckets[2][0], id);
        assert_eq!(buckets.buckets[8][0], id);
        assert_eq!(buckets.buckets[10][0], id);
        assert_eq!(buckets.buckets.len(), 16);
    }

    #[test]
    fn cross_boundary_remove() {
        use crate::physics::*;

        let world = World::new();
        let id = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb = AABB::new(0.0, 0.0, 10.0, 10.0);
        buckets.insert(id, &Transform::new(5.0, 5.0), &aabb);

        assert_eq!(buckets.buckets[0][0], id);
        assert_eq!(buckets.buckets[2][0], id);
        assert_eq!(buckets.buckets[8][0], id);
        assert_eq!(buckets.buckets[10][0], id);
        assert_eq!(buckets.buckets.len(), 16);

        buckets.remove(id, &Transform::new(5.0, 5.0), &aabb);

        assert_eq!(buckets.buckets[0].len(), 0);
        assert_eq!(buckets.buckets[2].len(), 0);
        assert_eq!(buckets.buckets[8].len(), 0);
        assert_eq!(buckets.buckets[10].len(), 0);
        assert_eq!(buckets.buckets.len(), 16);
    }

    #[test]
    fn multi_add_single_remove() {
        use crate::physics::*;

        let world = World::new();
        let id1 = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let id2 = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);

        buckets.insert(
            id1, 
            &Transform::new(5.0, 5.0),
            &AABB::new(0.0, 0.0, 10.0, 10.0),
        );

        buckets.insert(
            id2, 
            &Transform::new(5.0, 5.0),
            &AABB::new(0.0, 0.0, 10.0, 10.0),
        );

        assert_eq!(buckets.buckets[0][0], id1);
        assert_eq!(buckets.buckets[2][0], id1);
        assert_eq!(buckets.buckets[8][0], id1);
        assert_eq!(buckets.buckets[10][0], id1);

        assert_eq!(buckets.buckets[0][1], id2);
        assert_eq!(buckets.buckets[2][1], id2);
        assert_eq!(buckets.buckets[8][1], id2);
        assert_eq!(buckets.buckets[10][1], id2);
        
        assert_eq!(buckets.buckets.len(), 16);

        buckets.remove(id1, &Transform::new(5.0, 5.0), &AABB::new(0.0, 0.0, 10.0, 10.0));

        assert_eq!(buckets.buckets[0][0], id2);
        assert_eq!(buckets.buckets[2][0], id2);
        assert_eq!(buckets.buckets[8][0], id2);
        assert_eq!(buckets.buckets[10][0], id2);
        
        assert_eq!(buckets.buckets.len(), 16);
    }

    #[test]
    fn insert_and_resize() {
        use crate::physics::*;

        let world = World::new();
        let id = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb1 = AABB::new(0.0, 0.0, 1.0, 1.0);
        buckets.insert(id, &Transform::new(5.0, 5.0), &aabb1);

        assert_eq!(buckets.buckets[0][0], id);
        assert_eq!(buckets.buckets.len(), 1);

        buckets.resize();

        assert_eq!(buckets.buckets[0][0], id);
        let mut first = true;
        for bucket in buckets.buckets.iter() {
            if first {
                first = false;
                continue;
            }
            assert_eq!(bucket.len(), 0);
        }
        assert_eq!(buckets.buckets.len(), 4);
    }

    #[test]
    fn oob_insert() {
        use crate::physics::*;

        let world = World::new();
        let id = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb1 = AABB::new(0.0, 0.0, 1.0, 1.0);
        buckets.insert(id, &Transform::new(45.0, 45.0), &aabb1);

        assert_eq!(buckets.buckets[8 * buckets.width + 8][0], id);
        assert_eq!(buckets.buckets.len(), 256);
    }

    #[test]
    fn oob_insert_remove() {
        use crate::physics::*;

        let world = World::new();
        let id = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb1 = AABB::new(0.0, 0.0, 1.0, 1.0);
        buckets.insert(id, &Transform::new(45.0, 45.0), &aabb1);

        assert_eq!(buckets.buckets[8 * buckets.width + 8][0], id);
        assert_eq!(buckets.buckets.len(), 256);

        buckets.remove(id, &Transform::new(45.0, 45.0), &aabb1);

        assert_eq!(buckets.buckets[8 * buckets.width + 8].len(), 0);
        assert_eq!(buckets.buckets.len(), 256);
    }

    #[test]
    fn oob_insert_remove_nearby() {
        use crate::physics::*;

        let world = World::new();
        let id = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb1 = AABB::new(0.0, 0.0, 1.0, 1.0);
        buckets.insert(id, &Transform::new(45.0, 45.0), &aabb1);

        assert_eq!(buckets.buckets[8 * buckets.width + 8][0], id);
        assert_eq!(buckets.buckets.len(), 256);

        buckets.remove(id, &Transform::new(45.0, 45.0), &aabb1);

        assert_eq!(buckets.buckets[8 * buckets.width + 8].len(), 0);
        assert_eq!(buckets.buckets.len(), 256);

        let ids = buckets.nearby(EntityId::dead(), &Transform::new(45.0, 45.0), &aabb1);
        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn double_move_nearby() {
        use crate::physics::*;

        let world = World::new();
        let id1 = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });
        let id2 = world.run(|mut entities: EntitiesViewMut| {
            entities.add_entity((), ())
        });

        println!("id1: {:?}, id2: {:?}", id1, id2);

        let mut buckets = SpatialBuckets::new(10.0, 10.0);
        let aabb1 = AABB::new(0.0, 0.0, 1.0, 1.0);
        let aabb2 = AABB::new(0.0, 0.0, 1.0, 1.0);
        let mut t1 = Transform::new(5.0, 5.0);
        let mut t2 = Transform::new(5.0, 5.0);
        
        buckets.insert(id1, &t1, &aabb1);
        buckets.insert(id2, &t2, &aabb2);

        buckets.remove(id2, &t2, &aabb2);

        t2.x += 20.0;
        t2.y += 20.0;

        buckets.insert(id2, &t2, &aabb2);

        let nearby = buckets.nearby(id1, &t1, &aabb1);
        assert_eq!(nearby.len(), 0);
    }
}