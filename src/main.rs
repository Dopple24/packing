const EPS: f32 = 0.000001;

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
    fn set_prev_rc(&mut self, prev: Box<Corner>) {
        self.prev = Some(prev);
    }
    fn set_prev(&mut self, prev: Corner) {
        self.prev = Some(Box::new(prev));
    }
    fn clean_up(&mut self) {
        let mut curr: Box<Corner> = Box::new(self.clone());
        let mut next: Option<Corner> = None;
        while curr.prev.is_some() {
            let prev = curr.prev.clone().unwrap();
            if let Some(next_y) = next.as_mut() {
                if prev.y >= curr.y && next_y.y >= curr.y {
                    next_y.set_prev_rc(prev);
                }
            }
            curr = curr.prev.clone().unwrap();
        }
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
    pub fn place(&mut self, x: f32, y: f32) {
        self.x = Some(x);
        self.y = Some(y);
        self.is_placed = true;
    }
    pub fn bounds(&self) -> BoundingBox {
        let mut top = 0.0;
        let mut right = 0.0;
        self.points.iter().for_each(|point| {
            if point.0 > top {
                top = point.0
            }
            if point.1 > right {
                right = point.1
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
            println!("bounding boxws do not overlap");
            return false;
        }
        println!("bounding boxws overlap");
        self.does_overlap_intersect(other)
    }

    fn does_overlap_intersect(&self, other: &Shape) -> bool {
        let mut prev_self = self.points[0];
        let mut prev_other = self.points[0];
        let mut result = false;
        if does_intersect(
            (prev_self, *self.points.last().unwrap()),
            (prev_other, *other.points.last().unwrap()),
        ) {
            println!(
                "Overlap, because vec1: {:?}, {:?} and vec2: {:?}, {:?}",
                prev_self,
                *self.points.last().unwrap(),
                prev_other,
                *other.points.last().unwrap()
            );
            result = true;
        }
        println!("Iteration started");
        other.points.iter().for_each(|foreign| {
            self.points.iter().for_each(|my| {
                if does_intersect((prev_self, *my), (prev_other, *foreign)) {
                    println!(
                        "Overlap, because vec1: {:?}, {:?} and vec2: {:?}, {:?}",
                        prev_self, *my, prev_other, *foreign
                    );
                    result = true;
                }
                prev_self = *my;
                prev_other = *foreign;
            });
        });
        result
    }
}

#[derive(Clone, Debug)]
struct Rectangle {
    pub height: f32,
    pub width: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub is_placed: bool,
}

impl Rectangle {
    pub fn new(width: f32, height: f32) -> Self {
        Rectangle {
            height,
            width,
            x: None,
            y: None,
            is_placed: false,
        }
    }

    pub fn does_overlap(&self, other: &Rectangle) -> bool {
        let x1 = self.x.unwrap();
        let y1 = self.y.unwrap();
        let x2 = other.x.unwrap();
        let y2 = other.y.unwrap();

        !(x1 >= x2 + other.width
            || x1 + self.width <= x2
            || y1 >= y2 + other.height
            || y1 + self.height <= y2)
    }

    pub fn does_overlap_iter(&self, rectangles: &Vec<&Rectangle>) -> bool {
        let x1 = self.x.unwrap();
        let y1 = self.y.unwrap();

        let self_left = x1;
        let self_right = x1 + self.width;
        let self_bottom = y1;
        let self_top = y1 + self.height;

        for other in rectangles {
            let x2 = other.x.unwrap();
            let y2 = other.y.unwrap();

            let other_left = x2;
            let other_right = x2 + other.width;
            let other_bottom = y2;
            let other_top = y2 + other.height;

            let separated = self_right <= other_left
                || self_left >= other_right
                || self_top <= other_bottom
                || self_bottom >= other_top;

            if !separated {
                return true;
            }
        }

        false
    }

    pub fn place(&mut self, x: f32, y: f32) {
        self.x = Some(x);
        self.y = Some(y);
        self.is_placed = true;
    }

    pub fn remove(&mut self) {
        self.x = None;
        self.y = None;
        self.is_placed = false;
    }

    pub fn rotate(&mut self) {
        let temp_width = self.width;
        self.width = self.height;
        self.height = temp_width;
    }

    pub fn does_fit(&self, corner: &Corner, max_width: f32) -> bool {
        let rx = self.x.unwrap();
        let ry = self.y.unwrap();
        let rw = self.width;

        // Must not exceed strip width
        if rx + rw > max_width {
            return false;
        }

        // Walk the full corner list and check for overlap with each implied column
        // Each segment from corner[i].x to corner[i+1].x has floor at corner[i].y
        // The rect overlaps a column if it extends into x range AND sits below the floor
        let mut c = corner;
        loop {
            if let Some(prev) = &c.prev {
                // Column spans prev.x .. c.x with floor at prev.y
                let col_x0 = prev.x;
                let col_x1 = c.x;
                let col_floor = prev.y;

                let x_overlap = rx < col_x1 && rx + rw > col_x0;
                let y_overlap = ry < col_floor; // rect bottom is below this column's floor

                if x_overlap && y_overlap {
                    return false;
                }
                c = prev;
            } else {
                break;
            }
        }
        true
    }
}

fn main() {
    println!(
        "{:?}",
        fit(
            10.0,
            3.0,
            &mut vec![
                // Small squares
                Rectangle::new(1.0, 1.0),
                Rectangle::new(2.0, 2.0),
                Rectangle::new(3.0, 3.0),
                Rectangle::new(5.0, 0.01),
                // Mixed aspect ratios
                Rectangle::new(7.5, 2.3),
                Rectangle::new(2.3, 7.5),
                Rectangle::new(9.1, 4.7),
                Rectangle::new(4.7, 9.1),
                // Floating point precision cases
                Rectangle::new(1.0000001, 1.0),
                Rectangle::new(1.0, 1.0000001),
                Rectangle::new(3.3333333, 3.3333333),
                // Irregular realistic sizes
                Rectangle::new(11.0, 9.16510550),
                Rectangle::new(6.7, 8.2),
                Rectangle::new(12.4, 3.8),
                Rectangle::new(4.2, 11.9),
                // Repeated sizes (important for packing collisions)
                Rectangle::new(3.0, 3.0),
                Rectangle::new(3.0, 3.0),
                Rectangle::new(3.0, 3.0),
                Rectangle::new(4.0, 4.0),
                Rectangle::new(4.0, 4.0),
            ],
        )
    );
}

fn fit(width: f32, max_height: f32, rectangles: &mut Vec<Rectangle>) -> Vec<Rectangle> {
    let mut last_corner: Corner = Corner::new(width, 0.0);
    last_corner.set_prev(Corner::new(0.0, 0.0));
    rectangles.iter().for_each(|rect| {
        if rect.width > width && rect.height > width {
            panic!("A rectangle not fitable found");
        }
    });
    rectangles.sort_by_key(|rect| (rect.width * rect.height) as u32);
    rectangles.reverse();
    loop {
        last_corner.clean_up();

        let next_corner = match get_corner_place(&mut last_corner, width) {
            Some(val) => val,
            None => break,
        };

        println!(
            "-----------current best corner is-----------\n{:?}",
            next_corner.prev()
        );

        let rect_clone = rectangles.clone();

        let mut placed_rects: Vec<&Rectangle> =
            rect_clone.iter().filter(|rect| rect.is_placed).collect();

        let mut filtered_rects: Vec<&mut Rectangle> = rectangles
            .iter_mut()
            .filter(|rect| !rect.is_placed)
            .collect();

        if filtered_rects.is_empty() {
            break;
        }
        let mut is_rect_placed = false;
        for (index, rect) in filtered_rects.iter_mut().enumerate() {
            println!("Trying to place rectangle {}", index);
            rect.place(next_corner.prev().unwrap().x, next_corner.prev().unwrap().y);
            if rect.does_overlap_iter(&placed_rects)
                || !rect.does_fit(&next_corner.prev().unwrap(), width)
            {
                println!("failed, rotating");
                rect.remove();
                rect.rotate();
                rect.place(next_corner.prev().unwrap().x, next_corner.prev().unwrap().y);
                if rect.does_overlap_iter(&placed_rects)
                    || !rect.does_fit(&next_corner.prev().unwrap(), width)
                {
                    rect.remove();
                    println!("failed to place {:?}", rect);
                }
            }
            if rect.is_placed {
                println!("success: {:?}", rect);
                is_rect_placed = true;

                let mut top_left = Corner::new(rect.x.unwrap(), rect.height + rect.y.unwrap());
                let mut top_right =
                    Corner::new(rect.width + rect.x.unwrap(), rect.height + rect.y.unwrap());
                let mut bot_right = Corner::new(rect.width + rect.x.unwrap(), rect.y.unwrap());
                if next_corner.prev().unwrap().prev().is_some() {
                    top_left.set_prev_rc(next_corner.prev().unwrap().prev().unwrap());
                }
                top_right.set_prev(top_left);
                bot_right.set_prev(top_right);
                next_corner.set_prev(bot_right);
                break;
            }
        }
        if !is_rect_placed {
            println!("FAILED to place any rectangle at \n{:?}", next_corner);
            let mut rect = Rectangle::new(
                next_corner.x - next_corner.prev().unwrap().x,
                next_corner.prev().unwrap().prev().unwrap().y - next_corner.prev().unwrap().y,
            );
            rect.place(next_corner.prev().unwrap().x, next_corner.prev().unwrap().y);
            let mut rect_clone = rect.clone();
            placed_rects.push(&rect_clone.clone());
            filtered_rects.push(&mut rect_clone);

            match rect.width {
                ..=0.0 => {
                    next_corner.set_prev_rc(next_corner.clone().prev().unwrap().prev().unwrap());
                    println!("next_corner is now: \n{:?}", next_corner);
                }
                _ => {
                    let mut top_right =
                        Corner::new(rect.width + rect.x.unwrap(), rect.height + rect.y.unwrap());
                    top_right.set_prev_rc(next_corner.prev().unwrap().prev().unwrap());
                    next_corner.set_prev(top_right);
                    println!("next_corner is now: \n{:?}", next_corner);
                }
            }
        }
    }

    rectangles.clone()
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

fn does_intersect(line: ((f32, f32), (f32, f32)), other: ((f32, f32), (f32, f32))) -> bool {
    let orientation = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    };

    let on_segment = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        c.0 >= a.0.min(b.0) && c.0 <= a.0.max(b.0) && c.1 >= a.1.min(b.1) && c.1 <= a.1.max(b.1)
    };

    let vector_line = line;
    let vector_other = other;

    let cross_a_b = orientation(vector_line.0, vector_line.1, vector_other.0);
    let cross_a_c = orientation(vector_line.0, vector_line.1, vector_other.1);

    let cross_other_a_b = orientation(vector_other.0, vector_other.1, vector_line.0);
    let cross_other_a_c = orientation(vector_other.0, vector_other.1, vector_line.1);

    // General case
    if cross_a_b * cross_a_c < 0.0 && cross_other_a_b * cross_other_a_c < 0.0 {
        return true;
    }

    // Collinear / edge cases
    if cross_a_b.abs() < EPS && on_segment(vector_line.0, vector_line.1, vector_other.0) {
        return true;
    }
    if cross_a_c.abs() < EPS && on_segment(vector_line.0, vector_line.1, vector_other.1) {
        return true;
    }
    if cross_other_a_b.abs() < EPS && on_segment(vector_other.0, vector_other.1, vector_line.0) {
        return true;
    }
    if cross_other_a_c.abs() < EPS && on_segment(vector_other.0, vector_other.1, vector_line.1) {
        return true;
    }

    false
}

fn new_vec(source: &(f32, f32), goal: &(f32, f32)) -> (f32, f32) {
    (goal.0 - source.0, goal.1 - source.1)
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
        ]
    }

    #[test]
    fn test_complex_overlaps() {
        let s = make_complex_test_shapes();

        // Bounding boxes overlap → should be TRUE
        assert!(s[0].does_overlap(&s[1]), "0 and 1 should overlap (AABB)");

        // Clearly separate
        assert!(!s[0].does_overlap(&s[2]), "0 and 2 should NOT overlap");
        assert!(!s[2].does_overlap(&s[3]), "2 and 3 should NOT overlap");

        // Slanted rectangle intersects diamond
        assert!(s[2].does_overlap(&s[4]), "2 and 4 should overlap");

        // Far away shape
        assert!(!s[0].does_overlap(&s[5]), "0 and 5 should NOT overlap");
    }

    #[test]
    fn test_false_positive_bounding_box() {
        let s = make_complex_test_shapes();

        let result = s[0].does_overlap(&s[6]);

        // This is the IMPORTANT one:
        // AABB says TRUE, but real geometry says FALSE
        assert!(
            result,
            "Bounding boxes overlap, even though shapes do not (expected AABB limitation)"
        );
    }

    #[test]
    fn test_symmetry_complex() {
        let s = make_complex_test_shapes();

        for i in 0..s.len() {
            for j in 0..s.len() {
                let a = s[i].does_overlap(&s[j]);
                let b = s[j].does_overlap(&s[i]);

                assert_eq!(a, b, "Overlap should be symmetric for {} and {}", i, j);
            }
        }
    }
}
