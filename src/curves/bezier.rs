use glam::{vec2, Vec2};

pub fn get_bezier(mut control_points: Vec<Vec2>) -> Vec<Vec2> {
    let mut output = Vec::new();

    create_bezier(&mut output, &mut control_points);

    output
}

fn create_bezier(output: &mut Vec<Vec2>, control_points: &mut Vec<Vec2>) {
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

fn subdivide(control_points: &[Vec2], l: &mut Vec<Vec2>, r: &mut Vec<Vec2>) {
    let count = control_points.len();
    let mut midpoints: Vec<Vec2> = control_points.to_vec();

    for i in 0..count {
        l[i] = midpoints[0];
        r[count - i - 1] = midpoints[count - i - 1];

        for j in 0..count - i - 1 {
            midpoints[j] = (midpoints[j] + midpoints[j + 1]) / Vec2::new(2.0, 2.0);
        }
    }
}

fn approximate(control_points: &[Vec2], output: &mut Vec<Vec2>) {
    let count = control_points.len();
    let mut l: Vec<Vec2> = vec![vec2(0.0, 0.0); count * 2 - 1];
    let mut r: Vec<Vec2> = vec![vec2(0.0, 0.0); count];

    subdivide(&control_points, &mut l, &mut r);

    l[count..(count * 2) - 1].clone_from_slice(&r[1..count]);

    output.push(control_points[0]);

    for i in 1..count - 1 {
        let index = 2 * i;
        let p =
            (l[index] * Vec2::new(2.0, 2.0) + l[index - 1] + l[index + 1]) * Vec2::new(0.25, 0.25);
        output.push(p);
    }
}

fn is_flat_enough(control_points: &[Vec2], tolerance_sq: f32) -> bool {
    for i in 1..control_points.len() - 1 {
        if (control_points[i - 1] - control_points[i] * vec2(2.0, 2.0) + control_points[i + 1])
            .length_squared()
            > tolerance_sq
        {
            return false;
        }
    }

    true
}

fn create_singlebezier(output: &mut Vec<Vec2>, control_points: Vec<Vec2>) {
    let count = control_points.len();
    const TOLERANCE: f32 = 0.5;
    const TOLERANCE_SQ: f32 = TOLERANCE * TOLERANCE;

    if count == 0 {
        return;
    }

    let mut to_flatten: Vec<Vec<Vec2>> = Vec::new();
    let mut free_buffers: Vec<Vec<Vec2>> = Vec::new();

    let last_control_point = control_points[count - 1];
    to_flatten.push(control_points);

    let mut left_child = vec![vec2(0.0, 0.0); count * 2 - 1];

    while !to_flatten.is_empty() {
        let mut parent = to_flatten.pop().unwrap();
        if is_flat_enough(&parent, TOLERANCE_SQ) {
            approximate(&parent, output);
            free_buffers.push(parent);
            continue;
        }

        let mut right_child = free_buffers
            .pop()
            .unwrap_or_else(|| vec![vec2(0.0, 0.0); count]);

        subdivide(&parent, &mut left_child, &mut right_child);

        // We re-use the buffer of the parent for one of the children, so that we save one allocation per iteration.
        parent[..count].clone_from_slice(&left_child[..count]);

        to_flatten.push(right_child);
        to_flatten.push(parent);
    }

    output.push(last_control_point);
}
