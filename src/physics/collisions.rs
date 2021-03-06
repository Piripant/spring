use physics::simulation::{DebugView, Vertex};
use physics::surface::Surface;
use nalgebra::Vector2;
use Vector;

const VERTEX_RADIUS: f64 = 0.005;

#[inline]
fn cross(a: Vector2<f64>, b: Vector2<f64>) -> f64 {
    a.x * b.y - a.y * b.x
}

#[inline]
pub fn distance_vector(vertex: &Vector2<f64>, a: &Vector2<f64>, b: &Vector2<f64>) -> f64 {
    cross(b - vertex, vertex - a)
}

#[inline]
fn get_crossing(a: &Vector, b: &Vector, c: &Vector, d: &Vector) -> bool {
    distance_vector(a, c, d).signum() != distance_vector(b, c, d).signum()
        && distance_vector(c, a, b).signum() != distance_vector(d, a, b).signum()
}

#[inline]
fn ray_intersect_seg(mut p: Vector, mut a: Vector, mut b: Vector) -> bool {
    use std;
    use std::f64;

    const ESP: f64 = 0.00001;

    if a.y > b.y {
        std::mem::swap(&mut a, &mut b);
    }

    if p.y == a.y || p.y == b.y {
        p.y += ESP;
    }

    if (p.y > b.y || p.y < a.y) || p.x > a.x.max(b.x) {
        false
    } else if p.x < a.x.min(b.x) {
        true
    } else {
        let m_red = if (a.x - b.x).abs() > f64::MIN_POSITIVE {
            (b.y - a.y) / (b.x - a.x)
        } else {
            f64::MAX
        };
        let m_blue = if (a.x - p.x).abs() > f64::MIN_POSITIVE {
            (p.y - a.y) / (p.x - a.x)
        } else {
            f64::MAX
        };
        m_blue >= m_red
    }
}

#[inline]
fn get_inside(quad: [&Vector; 4], p: &Vector) -> bool {
    let p = p.clone();

    let mut count = 0;

    for i in 0..quad.len() {
        let ni = (i + 1) % quad.len();
        let mut a = quad[i].clone();
        let mut b = quad[ni].clone();

        if ray_intersect_seg(p, a, b) {
            count += 1;
        }
    }

    count % 2 == 1
}

#[inline]
fn get_colliding_poly(quad: [&Vector; 4], segment: [&Vector; 2]) -> bool {
    for i in 0..quad.len() {
        let ni = (i + 1) % quad.len();

        if get_crossing(segment[0], segment[1], quad[i], quad[ni]) {
            return true;
        }
    }

    for i in 0..2 {
        if get_inside(quad, segment[i]) {
            return true;
        }
    }

    false
}

pub fn colliding(vertex: &Vertex, a: &Vertex, b: &Vertex, dt: f64) -> bool {
    let quad = [
        &a.position,
        &b.position,
        &b.next_position(dt),
        &a.next_position(dt),
    ];

    let segment = [&vertex.position, &vertex.next_position(dt)];

    let colliding_poly = get_colliding_poly(quad, segment);
    let colliding_segment = distance_vector(&vertex.position, &a.position, &b.position).abs()
        < VERTEX_RADIUS
        && inside_box(&vertex.position, &a.position, &b.position);
    let colliding_vertex = (vertex.position - a.position).norm() < VERTEX_RADIUS
        || (vertex.position - b.position).norm() < VERTEX_RADIUS;

    colliding_poly || colliding_segment || colliding_vertex
}

/// Resolves the impulses between a `vertex` and a segment `ab`
pub fn resolve_impulses(vertex: &mut Vertex, a: &mut Vertex, b: &mut Vertex, surface: &Surface) {
    let e = surface.restitution as f64;

    let normal = normal(&vertex.position, &a.position, &b.position);

    let segment_mass = a.mass + b.mass;
    let segment_vel =
        (a.velocity * a.mass as f64 + b.velocity * b.mass as f64) / segment_mass as f64;

    let normal_velocity = (vertex.velocity - segment_vel).dot(&normal);

    // Resolve the collision bounce
    let j = -(1.0 + e) * normal_velocity / (1.0 / vertex.mass + 1.0 / segment_mass) as f64;
    let impulse = j * normal;

    let distance_ratio_ab =
        (b.position - vertex.position).norm() / (a.position - b.position).norm();
    let distance_ratio_ba = 1.0 - distance_ratio_ab;

    // Resolve the collision friction
    let tangent = tangent(&vertex.velocity, &a.position, &b.position);
    let tangent_velocity = (vertex.velocity - segment_vel).dot(&tangent);
    let coeff = surface.friction as f64;

    // Make the friction only stop the body and not make it go backwards
    let friction = {
        tangent * if normal_velocity.abs() * coeff < tangent_velocity.abs() {
            normal_velocity.abs() * coeff
        } else {
            tangent_velocity
        }
    };

    // Assign the new after-collision velocities
    vertex.velocity += impulse / vertex.mass as f64 - friction;
    a.velocity += -impulse * distance_ratio_ab / a.mass as f64 + friction;
    b.velocity += -impulse * distance_ratio_ba / b.mass as f64 + friction;

    if !vertex.is_static {
        vertex.position += normal * VERTEX_RADIUS / 2.0;
    }
    if !a.is_static {
        a.position -= normal * VERTEX_RADIUS / 2.0;
    }
    if !b.is_static {
        b.position -= normal * VERTEX_RADIUS / 2.0;
    }
}

#[inline]
pub fn normal(vertex: &Vector2<f64>, a: &Vector2<f64>, b: &Vector2<f64>) -> Vector2<f64> {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let normal = Vector2::new(-dy, dx).normalize();

    let distance = distance_vector(vertex, a, b);

    if distance <= 0.0 {
        normal
    } else {
        -normal
    }
}

#[inline]
pub fn tangent(direction: &Vector2<f64>, a: &Vector2<f64>, b: &Vector2<f64>) -> Vector2<f64> {
    let tangent = (a - b).normalize();
    let dot = direction.dot(&tangent);

    if dot > 0.0 {
        tangent
    } else {
        -tangent
    }
}

#[inline]
pub fn inside_box(vertex: &Vector2<f64>, a: &Vector2<f64>, b: &Vector2<f64>) -> bool {
    use std::f64;

    let max_x = f64::max(a.x, b.x);
    let min_x = f64::min(a.x, b.x);
    let max_y = f64::max(a.y, b.y);
    let min_y = f64::min(a.y, b.y);

    if vertex.x <= max_x && vertex.x >= min_x && vertex.y <= max_y && vertex.y >= min_y {
        return true;
    }

    false
}
