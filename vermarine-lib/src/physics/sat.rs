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
        if (self.min >= other.min && self.min <= other.max) || 
            (self.max >= other.min && self.max <= other.max) || 
            (self.max >= other.max && self.min <= other.min) {
            return true;
        }
        false
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