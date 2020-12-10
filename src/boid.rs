use nannou::prelude::*;
use crate::quadtree::*;

const MAXFORCE: f32 = 0.06;
const MAXSPEED: f32 = 2.5;

#[derive(Clone, PartialEq)]
pub(crate) struct Boid {
    pub position: Point2,
    pub velocity: Vector2,
    pub acceleration: Vector2,
    pub r: f32,
    pub highlight: bool,
}

impl Position for Boid {
    fn pos(&self) -> Point2 {
        self.position
    }
}

impl Boid {
    pub fn new(x: f32, y: f32) -> Self {
        let position = pt2(x, y);
        let velocity = vec2(0.0, 0.0);
        let acceleration = vec2(0.0, 0.0);
        let r = 2.0;
        let highlight = false;

        Boid {
            position,
            velocity,
            acceleration,
            r,
            highlight,
        }
    }

    pub fn accumualte(
        &self,
        boids: &Vec<Boid>,
        acc: impl Fn(&Boid) -> Vector2,
        steer: impl Fn(Vector2, f32) -> Vector2,
    ) -> Vector2 {
        let mut sum = vec2(0.0, 0.0);
        let mut count = 0.0;
        for b in boids {
            if b != self {
                sum += acc(b);
                count += 1.0;
            }
        }
        if count > 0.0 {
            return steer(sum, count);
        }
        vec2(0.0, 0.0)
    }

    pub fn align(&self, boids: &Vec<Boid>) -> Vector2 {
        let steer = |s: Vector2, c: f32| {
            ((s / c).with_magnitude(MAXSPEED) - self.velocity).limit_magnitude(MAXFORCE)
        };
        self.accumualte(boids, &|b: &Boid| b.velocity, &steer)
    }

    pub fn separate(&self, boids: &Vec<Boid>, dist: f32) -> Vector2 {
        let acc = |b: &Boid| (self.position - b.position).with_magnitude(1. / dist);
        let steer = |s: Vector2, _c: f32| {
            if s.magnitude() > 0. {
                return (s.with_magnitude(MAXSPEED) - self.velocity).limit_magnitude(MAXFORCE);
            } else {
                return vec2(0., 0.);
            }
        };
        self.accumualte(boids, &acc, &steer)
    }

    pub fn cohesion(&self, boids: &Vec<Boid>) -> Vector2 {
        let steer = |s: Vector2, c: f32| self.seek(s / c);
        self.accumualte(boids, &|b: &Boid| b.position, &steer)
    }

    pub fn update(&mut self) {
        self.velocity += self.acceleration;
        self.velocity.limit_magnitude(MAXSPEED);
        self.position += self.velocity;
        self.acceleration = vec2(0.0, 0.0);
    }

    fn seek(&self, target: Vector2) -> Vector2 {
        let desired = target - self.position;
        let desired = desired.with_magnitude(MAXSPEED);
        let steer = desired - self.velocity;
        steer.limit_magnitude(MAXFORCE)
    }

    pub fn borders(&mut self, win: &nannou::prelude::Rect) {
        let l = win.left();
        let r = win.right();
        let t = win.top();
        let b = win.bottom();
        let rd = self.r;
        match self.position {
            Vector2 { x, .. } if x < l - rd => self.position.x = r + rd,
            Vector2 { y, .. } if y < b - rd => self.position.y = t + rd,
            Vector2 { x, .. } if x > r + rd => self.position.x = l - rd,
            Vector2 { y, .. } if y > t + rd => self.position.y = b - rd,
            _ => (),
        };
    }
}
