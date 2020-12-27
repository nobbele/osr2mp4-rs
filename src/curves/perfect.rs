use glam::{vec2, Vec2};

const TOLERANCE: f32 = 0.1;

pub fn get_perfect(control_points: Vec<Vec2>) -> Vec<Vec2> {
    let a = control_points[0];
    let b = control_points[1];
    let c = control_points[2];

    let a_sq = (b - c).length_squared();
    let b_sq = (a - c).length_squared();
    let c_sq = (a - b).length_squared();

    // If we have a degenerate triangle where a side-length is almost zero, then give up and fall
    // back to a more numerically stable method.
    if a_sq == 0.0 || b_sq == 0.0 || c_sq == 0.0 {
        return Vec::new();
    }

    let s = a_sq * (b_sq + c_sq - a_sq);
    let t = b_sq * (a_sq + c_sq - b_sq);
    let u = c_sq * (a_sq + b_sq - c_sq);

    let sum = s + t + u;

    // If we have a degenerate triangle with an almost-zero size, then give up and fall
    // back to a more numerically stable method.
    if sum == 0.0 {
        return Vec::new();
    }

    let center = (s * a + t * b + u * c) / sum;
    let d_a = a - center;
    let d_c = c - center;

    let r = d_a.length();

    let theta_start = d_a.y.atan2(d_a.x);
    let mut theta_end = d_c.y.atan2(d_c.x);

    while theta_end < theta_start {
        theta_end += 2.0 * std::f32::consts::PI;
    }

    let mut dir = 1;
    let mut theta_range = theta_end - theta_start;

    // Decide in which direction to draw the circle, depending on which side of
    // AC B lies.
    let ortho_a_to_c = {
        let n = c - a;
        vec2(n.y, -n.x)
    };

    if (ortho_a_to_c).dot(b - a) < 0.0 {
        dir = -dir;
        theta_range = 2.0 * std::f32::consts::PI - theta_range;
    }

    // We select the amount of points for the approximation by requiring the discrete curvature
    // to be smaller than the provided tolerance. The exact angle required to meet the tolerance
    // is: 2 * Math.Acos(1 - TOLERANCE / r)
    // The special case is required for extremely short sliders where the radius is smaller than
    // the tolerance. This is a pathological rather than a realistic case.
    let amount_points = if 2.0 * r <= TOLERANCE {
        2
    } else {
        (theta_range / ((1.0 - TOLERANCE / r).acos() * 2.0))
            .ceil()
            .max(2.0) as usize
    };

    (0..amount_points)
        .map(|i| {
            let fract = i as f32 / (amount_points - 1) as f32;
            let theta = theta_start + dir as f32 * fract * theta_range;
            let o = vec2(theta.cos(), theta.sin()) * vec2(r, r);
            center + o
        })
        .collect::<Vec<_>>()
}
