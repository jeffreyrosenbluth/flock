mod boid;
mod quadtree;

use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou::Draw;

use crate::boid::*;
use crate::quadtree::*;

const WIDTH: u32 = 1500;
const HEIGHT: u32 = 1000;
const COUNT: usize = 600;
const CIRCLE: f32 = 200.0;
const SEPSTRENGTH: f32 = 1.5;
const SEPRADIUS: f32 = 25.0;
const ALISTRENGTH: f32 = 1.0;
const ALIRADIUS: f32 = 75.0;
const COHSTRENGTH: f32 = 1.0;
const COHRADIUS: f32 = 100.0;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    boids: Vec<Boid>,
    qtree: Box<QNode<Boid>>,
    ui: Ui,
    ids: Ids,
    sep_strength: f32,
    sep_radius: f32,
    ali_strength: f32,
    ali_radius: f32,
    coh_strength: f32,
    coh_radius: f32,
    grid: bool,
    trail: bool,
}

widget_ids! {
    struct Ids {
        sep_strength,
        sep_radius,
        ali_strength,
        ali_radius,
        coh_strength,
        coh_radius,
        reset,
        grid,
        fps,
        count,
        trail,
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
        b.velocity = vec2(z * x, -z * y).with_magnitude(1.0);
        boids.push(b);
        theta += delta;
    }
    boids
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(WIDTH, HEIGHT)
        .view(view)
        .build()
        .unwrap();
    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids::new(ui.widget_id_generator());

    let mut boids = boids_circle(COUNT, CIRCLE);
    boids[0].highlight = true;

    let qtree = Box::new(QNode::Points(vec![]));
    let sep_strength = SEPSTRENGTH;
    let sep_radius = SEPRADIUS;
    let ali_strength = ALISTRENGTH;
    let ali_radius = ALIRADIUS;
    let coh_strength = COHSTRENGTH;
    let coh_radius = COHRADIUS;
    let grid = false;
    let trail = false;

    Model {
        boids,
        qtree,
        ui,
        ids,
        sep_strength,
        sep_radius,
        ali_strength,
        ali_radius,
        coh_strength,
        coh_radius,
        grid,
        trail,
    }
}

fn update(app: &App, m: &mut Model, _update: Update) {
    let ui = &mut m.ui.set_widgets();

    fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, min, max)
            .w_h(150.0, 24.0)
            .label_font_size(12)
            .rgb(0.29, 0.53, 0.64)
            .label_rgb(0.83, 0.83, 0.85)
            .border(0.5)
            .border_rgb(37. / 255., 0.15, 0.15)
    }

    let count_label = format!("Boid Count {:.0}", m.boids.len());
    for value in slider(m.boids.len() as f32, 0.0, 2000.0)
        .top_left_with_margin(20.0)
        .label(&count_label[..])
        .set(m.ids.count, ui)
    {
        let bl = app.window_rect().bottom_left();
        let tr = app.window_rect().top_right();
        let v = value as usize;
        if v < m.boids.len() {
            m.boids.truncate(v);
        } else {
            for _ in 0..(v - m.boids.len()) {
                let x = random_range(bl.x, tr.x);
                let y = random_range(bl.y, tr.y);
                m.boids.push(Boid::new(x, y));
            }
        }
        if !m.boids.is_empty() {
            m.boids[0].highlight = true;
        }
    }

    let sep_label = format!("Separation Strength {:.1}", m.sep_strength);
    for value in slider(m.sep_strength, 0.0, 3.0)
        .down(10.0)
        .label(&sep_label[..])
        .set(m.ids.sep_strength, ui)
    {
        m.sep_strength = value;
    }

    let sep_label = format!("Separation Radius {:.0}", m.sep_radius);
    for value in slider(m.sep_radius, 0.0, 200.0)
        .down(10.0)
        .label(&sep_label[..])
        .set(m.ids.sep_radius, ui)
    {
        m.sep_radius = value;
    }

    let ali_label = format!("Alignment Strength {:.1}", m.ali_strength);
    for value in slider(m.ali_strength, 0.0, 3.0)
        .down(10.0)
        .label(&ali_label[..])
        .set(m.ids.ali_strength, ui)
    {
        m.ali_strength = value;
    }

    let ali_label = format!("Alignment Radius {:.0}", m.ali_radius);
    for value in slider(m.ali_radius, 0.0, 200.0)
        .down(10.0)
        .label(&ali_label[..])
        .set(m.ids.ali_radius, ui)
    {
        m.ali_radius = value;
    }

    let coh_label = format!("Cohesion Strength {:.1}", m.coh_strength);
    for value in slider(m.coh_strength, 0.0, 3.0)
        .down(10.0)
        .label(&coh_label[..])
        .set(m.ids.coh_strength, ui)
    {
        m.coh_strength = value;
    }

    let coh_label = format!("Cohesion Radius {:.0}", m.coh_radius);
    for value in slider(m.coh_radius, 0.0, 200.0)
        .down(10.0)
        .label(&coh_label[..])
        .set(m.ids.coh_radius, ui)
    {
        m.coh_radius = value;
    }

    for _click in widget::Button::new()
        .down(20.0)
        .w_h(150.0, 30.0)
        .label("Reset")
        .label_font_size(12)
        .rgb(0.15, 0.15, 0.15)
        .label_rgb(0.83, 0.83, 0.85)
        .border(0.0)
        .set(m.ids.reset, ui)
    {
        m.sep_strength = 1.5;
        m.sep_radius = 25.0;
        m.ali_strength = 1.0;
        m.ali_radius = 75.0;
        m.coh_strength = 1.0;
        m.coh_radius = 100.0;
        m.boids = boids_circle(m.boids.len(), 200.0);
        m.boids[0].highlight = true;
    }

    let grid_label = if m.grid { "Grid Off" } else { "Grid On" };
    for _click in widget::Button::new()
        .down(10.0)
        .w_h(150.0, 30.0)
        .label(grid_label)
        .label_font_size(12)
        .rgb(0.15, 0.15, 0.15)
        .label_rgb(0.83, 0.83, 0.85)
        .border(0.0)
        .set(m.ids.grid, ui)
    {
        m.grid = !m.grid
    }

    let trail_label = if m.trail { "Trail Off" } else { "Trail On" };
    for _click in widget::Button::new()
        .down(10.0)
        .w_h(150.0, 30.0)
        .label(trail_label)
        .label_font_size(12)
        .rgb(0.15, 0.15, 0.15)
        .label_rgb(0.83, 0.83, 0.85)
        .border(0.0)
        .set(m.ids.trail, ui)
    {
        m.trail = !m.trail
    }

    let fps_label = format!("fps {:.0}", app.fps().min(60.0));
    let _frame_rate = widget::TextBox::new(&fps_label[..])
        .bottom_left_with_margin(20.0)
        .w_h(150.0, 30.0)
        .font_size(12)
        .text_color(color::Color::Rgba(0.83, 0.83, 0.85, 1.0))
        .rgb(0.0, 0.0, 0.0)
        .set(m.ids.fps, ui);

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
    if m.trail {
        draw.rect()
            .wh(app.window_rect().wh())
            .color(srgba(0., 0., 0., 0.05));
    } else {
        draw.background().color(BLACK);
    }

    if m.grid {
        draw_qtree(m.qtree.clone(), bl, tr, &draw);
    }
    for boid in &m.boids {
        display(&boid, &draw, &m);
    }
    draw.to_frame(app, &frame).unwrap();
    m.ui.draw_to_frame(app, &frame).unwrap();
}

fn centered_rect(bl: Point2, tr: Point2) -> (Point2, Point2) {
    ((bl + tr) / 2.0, tr - bl)
}

fn draw_rect(bl: Point2, tr: Point2, draw: &Draw) {
    let (ctr, dims) = centered_rect(bl, tr);
    let clear = srgba(0.0, 0.0, 0.0, 0.0);
    draw.rect()
        .xy(ctr)
        .wh(dims)
        .color(clear)
        .stroke_color(rgb8(37, 38, 39))
        .stroke_weight(1.0);
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

fn display(boid: &Boid, draw: &Draw, m: &Model) {
    let Boid {
        position,
        velocity,
        highlight,
        ..
    } = boid;

    let theta = velocity.angle() + PI / 2.;
    let mut c = PLUM;
    let r = if m.trail { 1.0 } else { 2.0 };
    let clear = srgba(0.0, 0.0, 0.0, 0.0);

    if *highlight && m.grid && !m.trail {
        c = WHITE;
        draw.ellipse()
            .color(clear)
            .w_h(m.coh_radius * 2., m.coh_radius * 2.)
            .xy(*position)
            .stroke_weight(0.5)
            .stroke_color(rgb8(211, 212, 217));

        draw.ellipse()
            .color(clear)
            .w_h(m.ali_radius * 2., m.ali_radius * 2.)
            .xy(*position)
            .stroke_weight(0.5)
            .stroke_color(rgb8(75, 136, 162));

        draw.ellipse()
            .color(clear)
            .w_h(m.sep_radius * 2., m.sep_radius * 2.)
            .xy(*position)
            .stroke_weight(0.5)
            .stroke_color(rgb8(252, 81, 48));
    }
    let points = vec![
        pt2(0., -r * 2.),
        pt2(-r, r * 2.),
        pt2(0., r),
        pt2(r, r * 2.),
    ];
    draw.polygon()
        .points(points)
        .xy(*position)
        .color(c)
        .rotate(theta);
}
