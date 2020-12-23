use std::ops::{Add, Div, Mul, Sub};

use crate::mint::Point2;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point(pub Point2<f32>);

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point2 { x, y })
    }

    pub fn length_squared(self) -> f32 {
        self.0.x.powi(2) + self.0.y.powi(2)
    }
}

impl Mul for Point {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.0.x * rhs.0.x, self.0.y * rhs.0.y)
    }
}

impl Div for Point {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.0.x / rhs.0.x, self.0.y / rhs.0.y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0.x + rhs.0.x, self.0.y + rhs.0.y)
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0.x - rhs.0.x, self.0.y - rhs.0.y)
    }
}

pub fn get_bezier(mut control_points: Vec<Point>) -> Vec<Point> {
    let mut output = Vec::new();

    create_bezier(&mut output, &mut control_points);

    output
}

fn create_bezier(output: &mut Vec<Point>, control_points: &mut Vec<Point>) {
    let mut last_index = 0;
    let mut i = 0;
    while i < control_points.len() {
        let multipart_segment =
            i < control_points.len() - 2 && (control_points[i] == control_points[i + 1]);
        if multipart_segment || i == control_points.len() - 1 {
            let sub = control_points[last_index..i + 1].to_vec();
            if sub.len() == 2 {
                //create_linear(output, sub);
                todo!();
            } else {
                create_singlebezier(output, sub);
            }
            if multipart_segment {
                i += 1;
            }
            last_index = i;
        }
        i += 1;
    }
}

fn subdivide(control_points: &Vec<Point>, l: &mut Vec<Point>, r: &mut Vec<Point>) {
    let count = control_points.len();
    let mut midpoints: Vec<Point> = control_points.clone();

    for i in 0..count {
        l[i] = midpoints[0];
        r[count - i - 1] = midpoints[count - i - 1];

        for j in 0..count - i - 1 {
            midpoints[j] = (midpoints[j] + midpoints[j + 1]) / Point::new(2.0, 2.0);
        }
    }
}

fn approximate(control_points: &Vec<Point>, output: &mut Vec<Point>, ) {
    let count = control_points.len();
    let mut l: Vec<Point> = vec![Point::new(0.0, 0.0); count * 2 - 1];
    let mut r: Vec<Point> = vec![Point::new(0.0, 0.0); count];

    subdivide(&control_points, &mut l, &mut r);

    for i in 0..count - 1 {
        l[count + i] = r[i + 1];
    }

    output.push(control_points[0]);

    for i in 1..count - 1 {
        let index = 2 * i;
        let p = (l[index] * Point::new(2.0, 2.0) + l[index - 1] + l[index + 1])
            * Point::new(0.25, 0.25);
        output.push(p);
    }
}

fn is_flat_enough(control_points: &Vec<Point>, tolerance_sq: f32) -> bool {
    for i in 1..control_points.len() - 1 {
        if (control_points[i - 1] - control_points[i] * Point::new(2.0, 2.0)
            + control_points[i + 1])
            .length_squared()
            > tolerance_sq
        {
            return false;
        }
    }

    true
}

fn create_singlebezier(output: &mut Vec<Point>, control_points: Vec<Point>) {
    let count = control_points.len();
    const TOLERANCE: f32 = 0.5;
    const TOLERANCE_SQ: f32 = TOLERANCE * TOLERANCE;
    let subdivision_buffer2: Vec<Point> = vec![Point::new(0.0, 0.0); count * 2 - 1];

    if count == 0 {
        return;
    }

    let mut to_flatten: Vec<Vec<Point>> = Vec::new();
    let mut free_buffers: Vec<Vec<Point>> = Vec::new();

    let last_control_point = control_points[count - 1];
    to_flatten.push(control_points);

    let mut left_child = subdivision_buffer2.clone();

    while !to_flatten.is_empty() {
        let mut parent = to_flatten.pop().unwrap();
        if is_flat_enough(&parent, TOLERANCE_SQ) {
            approximate(&parent, output);
            free_buffers.push(parent);
            continue;
        }

        let mut right_child = free_buffers
            .pop()
            .unwrap_or_else(|| vec![Point::new(0.0, 0.0); count]);

        subdivide(&parent, &mut left_child, &mut right_child);

        // We re-use the buffer of the parent for one of the children, so that we save one allocation per iteration.
        for i in 0..count {
            parent[i] = left_child[i];
        }

        to_flatten.push(right_child);
        to_flatten.push(parent);
    }

    output.push(last_control_point);
}
