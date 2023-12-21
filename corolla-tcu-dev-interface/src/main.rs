use gtk4::cairo;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
use plotters::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

#[derive(Clone)]
struct GraphFunction {
    func: Rc<dyn Fn(f64) -> f64>,
    label: String,
    color: RGBColor,
}

fn main() {
    let application = create_application();
    application.run();
}

fn create_application() -> Application {
    let application = Application::builder()
        .application_id("com.github.gtk-rs.examples.basic")
        .build();

    application.connect_activate(|app| {
        let window = create_window(app);
        window.show();
    });

    application
}

fn create_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Line Graph")
        .default_width(800)
        .default_height(600)
        .build();

    let functions = create_functions();
    let graph_values = create_graph_values(&functions);
    let start_time = Rc::new(Instant::now());

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    window.set_child(Some(&vbox));

    for i in 0..functions.len() {
        let drawing_area = DrawingArea::new();
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);
        vbox.append(&drawing_area);
        set_timeout(&start_time, &graph_values[i], &drawing_area, &functions[i]);
        set_draw_func(&start_time, &graph_values[i], &drawing_area, &functions[i]);
    }
    
    window
}

fn create_functions() -> Vec<GraphFunction> {
    vec![
        GraphFunction { func: Rc::new(f64::sin), label: "y = sin(x)".to_string(), color: RED },
        GraphFunction { func: Rc::new(f64::cos), label: "y = cos(x)".to_string(), color: BLUE },
        GraphFunction { func: Rc::new(f64::tan), label: "y = tan(x)".to_string(), color: GREEN },
    ]
}

fn create_graph_values(functions: &Vec<GraphFunction>) -> Vec<Rc<RefCell<Vec<(f64, f64)>>>> {
    let mut graph_values = Vec::new();
    for _ in functions {
        graph_values.push(Rc::new(RefCell::new(Vec::new())));
    }
    graph_values
}

fn set_timeout(start_time: &Rc<Instant>, graph_value: &Rc<RefCell<Vec<(f64, f64)>>>, drawing_area: &DrawingArea, function: &GraphFunction) {
    let start_time = start_time.clone();
    let graph_value = Rc::clone(graph_value);
    let drawing_area = drawing_area.clone();
    let function = function.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(25), move || {
        let elapsed = start_time.elapsed().as_secs_f64();
        graph_value.borrow_mut().push((elapsed, (function.func)(elapsed * std::f64::consts::PI)));
        drawing_area.queue_draw();
        glib::ControlFlow::Continue
    });
}

fn set_draw_func(start_time: &Rc<Instant>, graph_value: &Rc<RefCell<Vec<(f64, f64)>>>, drawing_area: &DrawingArea, function: &GraphFunction) {
    let start_time = Rc::clone(start_time);
    let graph_value = Rc::clone(graph_value);
    let function = function.clone();

    drawing_area.set_draw_func(move |_, cr, width, height| {
        if width > 0 && height > 0 {
            render_graph(cr, width, height, start_time.clone(), &*graph_value.borrow(), &function, 40.0);
        }
    });
}

fn render_graph(cr: &cairo::Context, width: i32, height: i32, start_time: Rc<Instant>, graph_value: &[(f64, f64)], function: &GraphFunction, window_size: f64) {
    let elapsed = start_time.elapsed().as_secs_f64();
    let backend = plotters_cairo::CairoBackend::new(cr, (width as u32, height as u32)).unwrap();
    let root = backend.into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption(&function.label, ("sans-serif", 24).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d((elapsed-window_size).max(0.0)..elapsed, -1.2..1.2)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            graph_value.iter().cloned(),
            &function.color,
        ))
        .unwrap()
        .label(&function.label)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &function.color));

    chart.configure_series_labels().draw().unwrap();
}
