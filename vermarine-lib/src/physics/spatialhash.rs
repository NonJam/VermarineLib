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

        while xmax >= self.width || ymax >= self.height {
            self.resize();
        }

        for x in xmin..=xmax {
            for y in ymin..=ymax {
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

        while xmax >= self.width || ymax >= self.height {
            self.resize();
        }

        for x in xmin..=xmax {
            for y in ymin..=ymax {
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

        while xmax >= self.width || ymax >= self.height {
            self.resize();
        }

        let mut nearby = vec![];
        for x in xmin..=xmax {
            for y in ymin..=ymax {
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

    pub fn point_to_cell(&self, x: f64, y: f64) -> (usize, usize) {
        let mut x = f64::floor(x / self.bucket_width) as isize;
        x *= 2;
        if x < 0 {
            x *= -1;
            x -= 1;
        }
        let x = x as usize;

        let mut y = f64::floor(y / self.bucket_height) as isize;
        y *= 2;
        if y < 0 {
            y *= -1;
            y -= 1;
        }
        let y = y as usize;

        (x, y)
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

        assert_eq!(buckets.buckets[0][0], id);
        assert_eq!(buckets.buckets[2][0], id);
        assert_eq!(buckets.buckets[4][0], id);
        assert_eq!(buckets.buckets[6][0], id);
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
        assert_eq!(buckets.buckets[4][0], id);
        assert_eq!(buckets.buckets[6][0], id);
        assert_eq!(buckets.buckets.len(), 16);

        buckets.remove(id, &Transform::new(5.0, 5.0), &aabb);

        assert_eq!(buckets.buckets[0].len(), 0);
        assert_eq!(buckets.buckets[2].len(), 0);
        assert_eq!(buckets.buckets[4].len(), 0);
        assert_eq!(buckets.buckets[6].len(), 0);
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
        assert_eq!(buckets.buckets[4][0], id1);
        assert_eq!(buckets.buckets[6][0], id1);

        assert_eq!(buckets.buckets[0][1], id2);
        assert_eq!(buckets.buckets[2][1], id2);
        assert_eq!(buckets.buckets[4][1], id2);
        assert_eq!(buckets.buckets[6][1], id2);
        
        assert_eq!(buckets.buckets.len(), 16);

        buckets.remove(id1, &Transform::new(5.0, 5.0), &AABB::new(0.0, 0.0, 10.0, 10.0));

        assert_eq!(buckets.buckets[0][0], id2);
        assert_eq!(buckets.buckets[2][0], id2);
        assert_eq!(buckets.buckets[4][0], id2);
        assert_eq!(buckets.buckets[6][0], id2);
        
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
        let aabb1 = AABB::new(0.0, 0.0, 3.0, 3.0);
        buckets.insert(id, &Transform::new(5.0, 5.0), &aabb1);

        assert_eq!(buckets.buckets[0][0], id);
        assert_eq!(buckets.buckets.len(), 1);

        buckets.resize();

        assert_eq!(buckets.buckets[0][0], id);
        assert_eq!(buckets.buckets[1].len(), 0);
        assert_eq!(buckets.buckets[2].len(), 0);
        assert_eq!(buckets.buckets[3].len(), 0);

        assert_eq!(buckets.buckets.len(), 4);
    }
}