mod boid;
mod quadtree;

use nannou::color::{white_point::D65, Alpha, Lab, Laba};
use nannou::prelude::*;
use nannou::Draw;
use lazy_static::lazy_static;

use crate::boid::*;
use crate::quadtree::*;

const WIDTH: u32 = 1500;
const HEIGHT: u32 = 1000;
const COUNT: usize = 1000;
const CIRCLE: f32 = 200.0;
const SEPSTRENGTH: f32 = 1.5;
const SEPRADIUS: f32 = 50.0;
const ALISTRENGTH: f32 = 1.0;
const ALIRADIUS: f32 = 75.0;
const COHSTRENGTH: f32 = 1.0;
const COHRADIUS: f32 = 100.0;

const FRAME_COUNT: usize = 273;

lazy_static! {
static ref COLORS: Vec<Srgb<u8>> = vec![
    srgb8(3, 7, 30),
    srgb8(55, 6, 23),
    srgb8(106, 4, 15),
    srgb8(157, 2, 8),
    srgb8(208, 0, 0),
    srgb8(220, 47, 2),
    srgb8(232, 93, 4),
    srgb8(244, 140, 6),
    srgb8(250, 163, 7),
    srgb8(255, 186, 8),
];}

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    boids: Vec<Boid>,
    qtree: Box<QNode<Boid>>,
    sep_strength: f32,
    sep_radius: f32,
    ali_strength: f32,
    ali_radius: f32,
    coh_strength: f32,
    coh_radius: f32,
}

impl Model {
    fn new(boids: Vec<Boid>) -> Self {
        Self {
            boids,
            qtree: Box::new(QNode::Points(vec![])),
            sep_strength: SEPSTRENGTH,
            sep_radius: SEPRADIUS,
            ali_strength: ALISTRENGTH,
            ali_radius: ALIRADIUS,
            coh_strength: COHSTRENGTH,
            coh_radius: COHRADIUS,
        }
    }
}

fn boids_circle(n: usize, radius: f32) -> Vec<Boid> {
    let mut boids = Vec::new();
    let delta = f32::PI() * 2.0 / n as f32;
    let mut theta = 0.0;
    for i in 0..n {
        let x = radius * theta.cos() as f32;
        let y = radius * theta.sin() as f32;
        let z = if i % 2 == 0 { 1.0 } else { -1.0 };
        let mut b = Boid::new(x, y);
        b.velocity = vec2(z * x, -z * y).limit_magnitude(1.0);
        boids.push(b);
        theta += delta;
    }
    boids
}

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::loop_ntimes(FRAME_COUNT));
    app.new_window()
        .size(WIDTH, HEIGHT)
        .view(view)
        .build()
        .unwrap();

    let mut boids = boids_circle(COUNT, CIRCLE);
    boids[0].highlight = true;
    Model::new(boids)
}

fn update(app: &App, m: &mut Model, _update: Update) {
    let bl = app.window_rect().bottom_left();
    let tr = app.window_rect().top_right();
    let mut sep = Vec::new();
    let mut ali = Vec::new();
    let mut coh = Vec::new();
    let quad_tree = &mut QNode::Points(vec![]);

    for b in &m.boids {
        quad_tree.insert(b.clone(), bl, tr);
    }

    m.qtree = Box::new(quad_tree.clone());

    for boid in &m.boids {
        let sep_flock = quad_tree.points_in_circle(bl, tr, boid.pos(), m.sep_radius);
        let ali_flock = quad_tree.points_in_circle(bl, tr, boid.pos(), m.ali_radius);
        let coh_flock = quad_tree.points_in_circle(bl, tr, boid.pos(), m.coh_radius);
        sep.push(boid.separate(&sep_flock, m.sep_radius) * m.sep_strength);
        ali.push(boid.align(&ali_flock) * m.ali_strength);
        coh.push(boid.cohesion(&coh_flock) * m.coh_strength);
    }

    for (i, boid) in m.boids.iter_mut().enumerate() {
        boid.acceleration += sep[i] + ali[i] + coh[i];
        boid.borders(&app.window_rect());
        boid.update();
    }
}

fn view(app: &App, m: &Model, frame: Frame) {
    let bl = app.window_rect().bottom_left();
    let tr = app.window_rect().top_right();
    let draw = app.draw();
    draw.background().color(BLACK);
    draw_qtree(m.qtree.clone(), bl, tr, &draw);
    draw.to_frame(app, &frame).unwrap();
    let file_path = gif_path(app, &frame);
    app.main_window().capture_frame(file_path);
}

fn centered_rect(bl: Point2, tr: Point2) -> (Point2, Point2) {
    ((bl + tr) / 2.0, tr - bl)
}

fn draw_rect(bl: Point2, tr: Point2, draw: &Draw) {
    let (ctr, dims) = centered_rect(bl, tr);
    let k = (dims.x / WIDTH as f32 * 100.0) as usize % 10;
    draw.rect().xy(ctr).wh(dims).color(COLORS[k]);
}

fn draw_qtree(qtree: Box<QNode<Boid>>, bl: Point2, tr: Point2, draw: &Draw) {
    match *qtree {
        QNode::Points(_) => draw_rect(bl, tr, draw),
        QNode::Quad(qs) => {
            let (a, b) = blq(bl, tr);
            draw_rect(a, b, draw);
            draw_qtree(qs.bl, a, b, draw);
            let (a, b) = brq(bl, tr);
            draw_rect(a, b, draw);
            draw_qtree(qs.br, a, b, draw);
            let (a, b) = tlq(bl, tr);
            draw_rect(a, b, draw);
            draw_qtree(qs.tl, a, b, draw);
            let (a, b) = trq(bl, tr);
            draw_rect(a, b, draw);
            draw_qtree(qs.tr, a, b, draw);
        }
    }
}

fn gif_path(app: &App, frame: &Frame) -> std::path::PathBuf {
    app.project_path()
        .expect("failed to locate `project_path`")
        .join(app.exe_name().unwrap())
        .join(format!("frame_{:03}", frame.nth()))
        .with_extension("png")
}

pub fn random_color() -> Alpha<Lab<D65, f32>, f32> {
    let l: f32 = random_range(0.0, 100.0);
    let a: f32 = random_range(-128.0, 127.0);
    let b: f32 = random_range(-128.0, 127.0);
    let o: f32 = random_range(0.5, 1.0);
    Laba::new(l, a, b, o)
}
