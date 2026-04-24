use std::f32::EPSILON;

#[derive(Clone, Debug)]
struct Corner {
    pub prev: Option<Box<Corner>>,
    pub x: f32,
    pub y: f32,
}

impl Corner {
    fn new(x: f32, y: f32) -> Self {
        Corner { prev: None, x, y }
    }
    fn set_prev_box_opt(&mut self, prev: Option<Box<Corner>>) {
        self.prev = prev;
    }
    fn set_prev_rc(&mut self, prev: Box<Corner>) {
        self.prev = Some(prev);
    }
    fn set_prev(&mut self, prev: Corner) {
        self.prev = Some(Box::new(prev));
    }
    fn clean_up(&mut self, shapes: &Vec<Shape>) {
        let mut curr: Box<Corner> = Box::new(self.clone());
        //println!("clean_up beggining {:?}", curr);
        let mut next: Option<Corner> = None;
        while curr.prev.is_some() {
            let prev = curr.prev.clone().unwrap();
            let mut is_inside = false;
            if let Some(next_y) = next.as_mut() {
                for shape in shapes {
                    if shape.contains_point_strict((next_y.x, next_y.y)) {
                        is_inside = true;
                        break;
                    }
                }
                if (prev.y >= curr.y && next_y.y >= curr.y) || is_inside {
                    next_y.set_prev_rc(prev);
                }
            }
            curr = curr.prev.clone().unwrap();
        }
        //println!("clean up ended {:?}", curr);
    }
    fn prev(&mut self) -> Option<Box<Corner>> {
        self.prev.clone()
    }
}

struct BoundingBox {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(Clone, Debug)]
struct Shape {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub is_placed: bool,
    pub points: Vec<(f32, f32)>,
}

impl Shape {
    pub fn new(points: Vec<(f32, f32)>) -> Shape {
        Shape {
            x: None,
            y: None,
            is_placed: false,
            points,
        }
    }
    fn remove(&mut self) {
        self.x = None;
        self.y = None;
        self.is_placed = false;
    }
    pub fn place(&mut self, x: f32, y: f32) {
        self.x = Some(x);
        self.y = Some(y);
        self.is_placed = true;
    }
    pub fn bounds(&self) -> BoundingBox {
        let mut top = 0.0;
        let mut right = 0.0;
        self.points.iter().for_each(|point| {
            if point.1 > top {
                top = point.1
            }
            if point.0 > right {
                right = point.0
            }
        });
        BoundingBox {
            left: self.x.unwrap(),
            right: right + self.x.unwrap(),
            top: top + self.y.unwrap(),
            bottom: self.y.unwrap(),
        }
    }

    fn does_overlap(&self, other: &Shape) -> bool {
        let a = self.bounds();
        let b = other.bounds();
        if a.right <= b.left || a.left >= b.right || a.bottom >= b.top || a.top <= b.bottom {
            //println!("bounding boxws do not overlap");
            return false;
        }
        //println!("bounding boxws overlap");
        self.does_overlap_intersect(other)
    }

    fn get_corners(&self, next: &mut Corner, prev: Option<Box<Corner>>) {
        //println!("next: {:?}, prev: {:?}", next, prev);
        let mut prev = prev;
        self.points.iter().for_each(|(x, y)| {
            let mut corn = Corner::new(*x + self.x.unwrap(), *y + self.y.unwrap());
            corn.set_prev_box_opt(prev.clone());
            prev = Some(Box::new(corn));
        });
        next.set_prev_box_opt(prev);
        //println!("next: {:?}", next);
    }

    fn does_overlap_intersect(&self, other: &Shape) -> bool {
        let mut prev_self = *self.points.last().unwrap();
        let mut prev_other = *self.points.last().unwrap();
        let mut result = false;
        if does_intersect(
            (prev_self, *self.points.last().unwrap()),
            (prev_other, *other.points.last().unwrap()),
        ) {
            /*//println!(
                "Overlap, because vec1: {:?}, {:?} and vec2: {:?}, {:?}",
                prev_self,
                *self.points.last().unwrap(),
                prev_other,
                *other.points.last().unwrap()
            );*/
            result = true;
        }
        other.points.iter().for_each(|point| {
            if self.contains_point_strict((
                (*point).0 + other.x.unwrap(),
                (*point).1 + other.y.unwrap(),
            )) {
                //println!("self contains this point: {:?}, self: {:?}", point, self);
                result = true;
            }
        });
        if result {
            return true;
        }
        self.points.iter().for_each(|point| {
            if other
                .contains_point_strict(((*point).0 + self.x.unwrap(), (*point).1 + self.y.unwrap()))
            {
                //println!("other contains this point: {:?}, other: {:?}", point, other);
                result = true;
            }
        });
        if result {
            return true;
        }
        let mut is_exactly_same = true;
        other.points.iter().for_each(|foreign| {
            let mut has_same = false;
            self.points.iter().for_each(|my| {
                /*//println!(
                    "points compared: {:?}, {:?} from shape: {:?} and {:?}, {:?} from shape: {:?}",
                    prev_self, *my, self, prev_other, *foreign, other,
                );*/
                if my.0 + self.x.unwrap() == foreign.0 + other.x.unwrap()
                    && my.1 + self.y.unwrap() == foreign.1 + other.y.unwrap()
                {
                    has_same = true;
                }
                if does_intersect(
                    (
                        (prev_self.0 + self.x.unwrap(), prev_self.1 + self.y.unwrap()),
                        ((*my).0 + self.x.unwrap(), (*my).1 + self.y.unwrap()),
                    ),
                    (
                        (
                            prev_other.0 + other.x.unwrap(),
                            prev_other.1 + other.y.unwrap(),
                        ),
                        (
                            (*foreign).0 + other.x.unwrap(),
                            (*foreign).1 + other.y.unwrap(),
                        ),
                    ),
                ) {
                    result = true;
                }
                //println!("result: {result}");
                prev_self = *my;
            });
            if !has_same {
                is_exactly_same = false;
            }
            prev_other = *foreign;
        });
        //println!("is_exactly_same: {is_exactly_same}");
        result || is_exactly_same
    }

    //GPT generated
    pub fn contains_point_strict(&self, point: (f32, f32)) -> bool {
        let (px, py) = point;
        let n = self.points.len();

        if n < 3 {
            return false;
        }

        // 1. Check if point lies exactly on any edge → OUTSIDE
        for i in 0..n {
            let (x1, y1) = self.points[i];
            let (x2, y2) = self.points[(i + 1) % n];

            if point_on_segment(px, py, x1, y1, x2, y2) {
                return false;
            }
        }

        // 2. Ray casting
        let mut inside = false;

        for i in 0..n {
            let (x1, y1) = (
                self.points[i].0 + self.x.unwrap(),
                self.points[i].1 + self.y.unwrap(),
            );
            let (x2, y2) = (
                self.points[(i + 1) % n].0 + self.x.unwrap(),
                self.points[(i + 1) % n].1 + self.y.unwrap(),
            );

            let intersects =
                ((y1 > py) != (y2 > py)) && (px < (x2 - x1) * (py - y1) / (y2 - y1) + x1);

            if intersects {
                inside = !inside;
            }
        }

        inside
    }
}

fn main() {
    fit(
        20.0,
        &mut vec![
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![
                    (0.0, 0.0),
                    (3.0, 0.0),
                    (3.0, 1.0),
                    (1.0, 1.0),
                    (1.0, 3.0),
                    (0.0, 3.0),
                ],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(1.5, 0.0), (3.0, 1.5), (1.5, 3.0), (0.0, 1.5)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![
                    (0.5, 0.0),
                    (2.5, 0.5),
                    (1.5, 1.5),
                    (3.0, 2.5),
                    (1.0, 3.0),
                    (0.0, 1.5),
                ],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (0.3, 0.2), (1.7, 3.8), (1.4, 3.6)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.5, 0.0), (3.5, 0.5), (4.5, 3.0), (2.0, 5.0), (0.0, 3.0)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![
                    (0.0, 0.0),
                    (3.0, 0.0),
                    (3.0, 1.0),
                    (1.0, 1.0),
                    (1.0, 3.0),
                    (0.0, 3.0),
                ],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(1.5, 0.0), (3.0, 1.5), (1.5, 3.0), (0.0, 1.5)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![
                    (0.5, 0.0),
                    (2.5, 0.5),
                    (1.5, 1.5),
                    (3.0, 2.5),
                    (1.0, 3.0),
                    (0.0, 1.5),
                ],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (0.3, 0.2), (1.7, 3.8), (1.4, 3.6)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.5, 0.0), (3.5, 0.5), (4.5, 3.0), (2.0, 5.0), (0.0, 3.0)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![
                    (0.0, 0.0),
                    (3.0, 0.0),
                    (3.0, 1.0),
                    (1.0, 1.0),
                    (1.0, 3.0),
                    (0.0, 3.0),
                ],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(1.5, 0.0), (3.0, 1.5), (1.5, 3.0), (0.0, 1.5)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![
                    (0.5, 0.0),
                    (2.5, 0.5),
                    (1.5, 1.5),
                    (3.0, 2.5),
                    (1.0, 3.0),
                    (0.0, 1.5),
                ],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (0.3, 0.2), (1.7, 3.8), (1.4, 3.6)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.5, 0.0), (3.5, 0.5), (4.5, 3.0), (2.0, 5.0), (0.0, 3.0)],
            },
            Shape {
                x: None,
                y: None,
                is_placed: false,
                points: vec![(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)],
            },
        ],
    );
}

fn fit(width: f32, shapes: &mut Vec<Shape>) {
    let mut corners = Corner::new(width, 0.0);
    let prev = Corner::new(0.0, 0.0);
    corners.set_prev(prev);
    shapes.sort_by_key(|shape| {
        let mut circ = 0.0;
        let mut prev = shape.points[0];
        shape.points.iter().for_each(|p| {
            circ += (p.0 - prev.0).powi(2) + (p.1 - prev.1).powi(2);
            prev = p.clone();
        });
        circ as usize
    });
    shapes.reverse();
    loop {
        corners.clean_up(shapes);

        let next_corner = match get_corner_place(&mut corners, width) {
            Some(val) => val,
            None => break,
        };

        /*//println!(
            "-----------current best corner's parent is-----------\n{:?}",
            next_corner
        );*/

        let placed: Vec<Shape> = shapes
            .clone()
            .into_iter()
            .filter(|sha| sha.is_placed)
            .collect();

        let mut filtered: Vec<&mut Shape> =
            shapes.iter_mut().filter(|shape| !shape.is_placed).collect();

        if filtered.is_empty() {
            break;
        }

        println!("{:?}", placed);

        let mut is_shape_placed: bool = false;

        for shape in filtered.iter_mut() {
            //println!("trying to place: {:?} at {:?}", shape, next_corner);
            shape.place(next_corner.prev().unwrap().x, next_corner.prev().unwrap().y);
            let mut does_overlap = false;
            for s in placed.iter() {
                if shape.does_overlap(s) {
                    //println!("overlaped");
                    shape.remove();
                    does_overlap = true;
                    break;
                }
            }
            if !does_overlap {
                is_shape_placed = true;
                //println!("doesn't overlap");
                shape.get_corners(next_corner, next_corner.clone().prev().unwrap().prev());
                break;
            }
        }
        if !is_shape_placed {
            let curr = next_corner.clone().prev().unwrap();
            let (mut width, mut height) = match next_corner.clone().prev().unwrap().prev() {
                Some(prev) => {
                    let width = if next_corner.x.min(prev.x) - curr.x < EPSILON {
                        next_corner.x.max(prev.x) - curr.x
                    } else {
                        next_corner.x.min(prev.x) - curr.x
                    };
                    let height = if next_corner.y.min(prev.y) - curr.y < EPSILON {
                        next_corner.y.min(prev.y) - curr.y
                    } else {
                        next_corner.y.min(prev.y) - curr.y
                    };
                    (width, height)
                }
                None => (next_corner.x - curr.x, next_corner.y - curr.y),
            };
            let mut virt = Shape::new(vec![
                (0.0, 0.0),
                (0.0, height),
                (width, height),
                (width, 0.0),
            ]);
            virt.place(curr.x, curr.y);
            if let Some(mut prev) = next_corner.clone().prev().unwrap().prev() {
                if (width < f32::EPSILON) && (height < f32::EPSILON) {
                } else if width < f32::EPSILON {
                    if let Some(pre_prev) = prev.prev() {
                        if pre_prev.x - curr.x > EPSILON {
                            width = (pre_prev.x - curr.x).max(0.5);
                        }
                    }
                } else if height < f32::EPSILON {
                    if let Some(pre_prev) = prev.prev() {
                        if pre_prev.y - curr.y > EPSILON {
                            height = (pre_prev.y - curr.y).max(0.5);
                        }
                    }
                }
            }
            let prev = next_corner.clone().prev().unwrap().prev();
            let mut up = Corner::new(curr.x, curr.y + height);
            let mut dia = Corner::new(curr.x + width, curr.y + height);
            let mut rig = Corner::new(curr.x + width, curr.y);

            if (width < f32::EPSILON) && (height < f32::EPSILON) {
                next_corner.set_prev_box_opt(prev);
            } else if width < f32::EPSILON {
                dia.set_prev_box_opt(prev);
                next_corner.set_prev(dia);
            } else if height < f32::EPSILON {
                dia.set_prev_box_opt(prev);
                next_corner.set_prev(dia);
            } else {
                up.set_prev_box_opt(prev);
                dia.set_prev(up);
                rig.set_prev(dia);
                next_corner.set_prev(rig);
            }
            filtered.push(&mut virt);
            //println!("creating virtual rectangle {:?}", virt);
        }
    }
}

///finds lowest corner (if multiple, than the leftiest one). Returns that corner's parent
fn get_corner_place(last_corner: &mut Corner, max_width: f32) -> Option<&mut Corner> {
    let mut curr: *mut Corner = last_corner;
    let mut prev: *mut Corner = last_corner;
    let mut best_parent: *mut Corner = std::ptr::null_mut();
    let mut lowest = f32::MAX;
    let mut leftmost = f32::MAX;

    loop {
        let curr_ref = unsafe { &*curr };

        // skip the right boundary sentinel
        if curr_ref.x < max_width {
            let is_better = curr_ref.y < lowest || (curr_ref.y == lowest && curr_ref.x < leftmost);

            if is_better {
                lowest = curr_ref.y;
                leftmost = curr_ref.x;
                best_parent = prev;
            }
        }

        match unsafe { &mut *curr }.prev.as_mut() {
            Some(next) => {
                prev = curr;
                curr = next.as_mut();
            }
            None => break,
        }
    }

    if best_parent.is_null() {
        None
    } else {
        unsafe { Some(&mut *best_parent) }
    }
}

// Helper: check if point lies on a line segment, Claude generated
fn point_on_segment(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
    let cross = (py - y1) * (x2 - x1) - (px - x1) * (y2 - y1);
    if cross.abs() > 1e-6 {
        return false;
    }
    let dot = (px - x1) * (px - x2) + (py - y1) * (py - y2);
    dot <= 0.0
}

fn does_intersect(line: ((f32, f32), (f32, f32)), other: ((f32, f32), (f32, f32))) -> bool {
    let epsilon = 1e-6;

    // Helper to check if two points are equal
    let points_equal =
        |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < epsilon && (a.1 - b.1).abs() < epsilon;

    let orientation = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    };

    let on_segment = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        c.0 >= a.0.min(b.0) && c.0 <= a.0.max(b.0) && c.1 >= a.1.min(b.1) && c.1 <= a.1.max(b.1)
    };

    if points_equal(line.0, line.1) || points_equal(other.0, other.1) {
        return false;
    }

    let vector_line = line;
    let vector_other = other;

    let cross_a_b = orientation(vector_line.0, vector_line.1, vector_other.0);
    let cross_a_c = orientation(vector_line.0, vector_line.1, vector_other.1);
    let cross_other_a_b = orientation(vector_other.0, vector_other.1, vector_line.0);
    let cross_other_a_c = orientation(vector_other.0, vector_other.1, vector_line.1);

    /*//println!("vectors: {:?}, {:?}", vector_line, vector_other);

    //println!(
        "cross_a_b: {:?}, cross_a_c: {:?}, cross_other_a_b: {:?}, cross_other_a_c: {:?}",
        cross_a_b, cross_a_c, cross_other_a_b, cross_other_a_c
    );*/

    // General case: segments properly cross (not at endpoints)
    if cross_a_b * cross_a_c < 0.0 && cross_other_a_b * cross_other_a_c < 0.0 {
        return true;
    }

    // Collinear cases: exclude if touching at endpoints only
    if cross_a_b.abs() < epsilon && on_segment(vector_line.0, vector_line.1, vector_other.0) {
        // Point is on segment, but exclude if it's an endpoint match
        if !points_equal(vector_other.0, vector_line.0)
            && !points_equal(vector_other.0, vector_line.1)
        {
            return true;
        }
    }

    if cross_a_c.abs() < epsilon && on_segment(vector_line.0, vector_line.1, vector_other.1) {
        // Point is on segment, but exclude if it's an endpoint match
        if !points_equal(vector_other.1, vector_line.0)
            && !points_equal(vector_other.1, vector_line.1)
        {
            return true;
        }
    }

    if cross_other_a_b.abs() < epsilon && on_segment(vector_other.0, vector_other.1, vector_line.0)
    {
        // Point is on segment, but exclude if it's an endpoint match
        if !points_equal(vector_line.0, vector_other.0)
            && !points_equal(vector_line.0, vector_other.1)
        {
            return true;
        }
    }

    if cross_other_a_c.abs() < epsilon && on_segment(vector_other.0, vector_other.1, vector_line.1)
    {
        // Point is on segment, but exclude if it's an endpoint match
        if !points_equal(vector_line.1, vector_other.0)
            && !points_equal(vector_line.1, vector_other.1)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_complex_test_shapes() -> Vec<Shape> {
        vec![
            // 0: Concave "L" shape (already valid)
            Shape {
                x: Some(0.0),
                y: Some(0.0),
                is_placed: true,
                points: vec![
                    (0.0, 0.0),
                    (3.0, 0.0),
                    (3.0, 1.0),
                    (1.0, 1.0),
                    (1.0, 3.0),
                    (0.0, 3.0),
                ],
            },
            // 1: Triangle
            Shape {
                x: Some(1.5),
                y: Some(1.5),
                is_placed: true,
                points: vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)],
            },
            // 2: Diamond (fixed: no negative coords)
            Shape {
                x: Some(4.5), // min x
                y: Some(2.0), // min y
                is_placed: true,
                points: vec![
                    (1.5, 0.0), // (6.0,2.0)
                    (3.0, 1.5), // (7.5,3.5)
                    (1.5, 3.0), // (6.0,5.0)
                    (0.0, 1.5), // (4.5,3.5)
                ],
            },
            // 3: Concave zig-zag (fixed)
            Shape {
                x: Some(7.5),
                y: Some(0.0),
                is_placed: true,
                points: vec![
                    (0.5, 0.0), // (8.0,0.0)
                    (2.5, 0.5), // (10.0,0.5)
                    (1.5, 1.5), // (9.0,1.5)
                    (3.0, 2.5), // (10.5,2.5)
                    (1.0, 3.0), // (8.5,3.0)
                    (0.0, 1.5), // (7.5,1.5)
                ],
            },
            // 4: Thin slanted rectangle (already valid)
            Shape {
                x: Some(5.5),
                y: Some(1.0),
                is_placed: true,
                points: vec![(0.0, 0.0), (0.3, 0.2), (1.7, 3.8), (1.4, 3.6)],
            },
            // 5: Pentagon (fixed)
            Shape {
                x: Some(14.5),
                y: Some(15.0),
                is_placed: true,
                points: vec![
                    (0.5, 0.0), // (15.0,15.0)
                    (3.5, 0.5), // (18.0,15.5)
                    (4.5, 3.0), // (19.0,18.0)
                    (2.0, 5.0), // (16.5,20.0)
                    (0.0, 3.0), // (14.5,18.0)
                ],
            },
            // 6: Small square (already valid)
            Shape {
                x: Some(2.0),
                y: Some(2.0),
                is_placed: true,
                points: vec![(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)],
            },
            //7
            Shape {
                x: Some(3.5),
                y: Some(0.0),
                is_placed: true,
                points: vec![(0.0, 0.0), (0.3, 0.2), (1.7, 3.8), (1.4, 3.6)],
            },
            //8
            Shape {
                x: Some(3.5), // min x
                y: Some(3.0), // min y
                is_placed: true,
                points: vec![
                    (1.5, 0.0), // (6.0,2.0)
                    (3.0, 1.5), // (7.5,3.5)
                    (1.5, 3.0), // (6.0,5.0)
                    (0.0, 1.5), // (4.5,3.5)
                ],
            },
        ]
    }

    #[test]
    fn test_complex_overlaps() {
        let s = make_complex_test_shapes();

        assert!(
            !s[0].does_overlap(&s[1]),
            "0 and 1 should NOT overlap (AABB)"
        );

        // Clearly separate
        assert!(!s[0].does_overlap(&s[2]), "0 and 2 should NOT overlap");
        assert!(!s[2].does_overlap(&s[3]), "2 and 3 should NOT overlap");

        // Slanted rectangle intersects diamond
        assert!(s[2].does_overlap(&s[4]), "2 and 4 should overlap");

        // Far away shape
        assert!(!s[0].does_overlap(&s[5]), "0 and 5 should NOT overlap");

        //println!("printing 7, 8");
        assert!(s[7].does_overlap(&s[8]), "7 and 8 should overlap");
    }

    #[test]
    fn test_false_positive_bounding_box() {
        let s = make_complex_test_shapes();

        let result = s[0].does_overlap(&s[6]);

        // This is the IMPORTANT one:
        // AABB says TRUE, but real geometry says FALSE
        assert!(
            !result,
            "Bounding boxes overlap, even though shapes do not (expected AABB limitation)"
        );
    }

    #[test]
    fn test_symmetry_complex() {
        let s = make_complex_test_shapes();

        assert_eq!(s[0].does_overlap(&s[1]), s[1].does_overlap(&s[0]));

        for i in 0..s.len() {
            for j in 0..s.len() {
                let a = s[i].does_overlap(&s[j]);
                let b = s[j].does_overlap(&s[i]);

                assert_eq!(a, b, "Overlap should be symmetric for {} and {}", i, j);
            }
        }
    }

    #[test]
    fn segment_intersection() {
        // Both originate at (0,0) → false
        assert!(!does_intersect(
            ((0.0, 0.0), (1.0, 0.0)),
            ((0.0, 0.0), (0.0, 1.0))
        )); // false

        // Same endpoint → false
        assert!(!does_intersect(
            ((0.0, 0.0), (1.0, 0.0)),
            ((0.0, 0.0), (1.0, 1.0))
        )); // false

        // Collinear but just touching ends → false
        assert!(!does_intersect(
            ((0.0, 0.0), (1.0, 0.0)),
            ((1.0, 0.0), (2.0, 0.0))
        )); // false

        // Actually crossing (not at endpoint) → true
        assert!(does_intersect(
            ((0.0, 0.0), (2.0, 2.0)),
            ((0.0, 2.0), (2.0, 0.0))
        )); // true

        // One segment crossing through middle of other → true
        assert!(does_intersect(
            ((0.0, 0.0), (2.0, 0.0)),
            ((1.0, -1.0), (1.0, 1.0))
        )); // true

        assert!(does_intersect(
            ((0.5, 0.0), (3.5, 0.5)),
            ((0.5, 3.0), (1.0, 0.0))
        )); // true
    }

    /*#[test]
    fn corner_addition() {
        let shapes = make_complex_test_shapes();
        let mut curr = Corner::new(1.0, 0.0);
        curr.set_prev(Corner::new(0.0, 0.0));
        let mut next = Corner::new(10.0, 10.0);
        let mut prev = shapes[0].get_corners(&mut next, curr.prev.unwrap());
        //println!("shape: {:?}", shapes[0]);
        //println!("{:?}", prev);
        assert!(false);
    }*/
}
