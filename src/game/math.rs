pub fn intersect_circle(x1: f32, y1: f32, x2: f32, y2: f32, r: f32) -> bool {
    let dx = x1 - x2;
    let dy = y1 - y2;
    dx * dx + dy * dy < r * r
}

#[allow(dead_code)]
pub fn distance_squared(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    dx * dx + dy * dy
}

pub fn normalize_angle(angle: f32) -> f32 {
    let two_pi = 2.0 * std::f32::consts::PI;
    let mut a = angle % two_pi;
    if a < 0.0 {
        a += two_pi;
    }
    a
}
