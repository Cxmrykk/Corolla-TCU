use gtk4::cairo;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
use plotters::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

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

    let drawing_area = DrawingArea::new();
    window.set_child(Some(&drawing_area));

    let functions = create_functions();
    let graph_values = create_graph_values(&functions);
    let start_time = Rc::new(Instant::now());

    set_timeout(&start_time, &graph_values, &drawing_area, &functions);
    set_draw_func(&start_time, &graph_values, &drawing_area, &functions);

    window
}

fn create_functions() -> Rc<Vec<GraphFunction>> {
    Rc::new(vec![
        GraphFunction { func: Rc::new(f64::sin), label: "y = sin(x)".to_string(), color: RED },
        GraphFunction { func: Rc::new(f64::cos), label: "y = cos(x)".to_string(), color: BLUE },
        GraphFunction { func: Rc::new(f64::tan), label: "y = tan(x)".to_string(), color: GREEN },
    ])
}

fn create_graph_values(functions: &Vec<GraphFunction>) -> Vec<Rc<RefCell<Vec<(f64, f64)>>>> {
    let mut graph_values = Vec::new();
    for _ in functions {
        graph_values.push(Rc::new(RefCell::new(Vec::new())));
    }
    graph_values
}

fn set_timeout(start_time: &Rc<Instant>, graph_values: &Vec<Rc<RefCell<Vec<(f64, f64)>>>>, drawing_area: &DrawingArea, functions: &Rc<Vec<GraphFunction>>) {
    let start_time = start_time.clone();
    let graph_values = graph_values.clone();
    let drawing_area = drawing_area.clone();
    let functions = Rc::clone(functions);

    glib::timeout_add_local(std::time::Duration::from_millis(25), move || {
        let elapsed = start_time.elapsed().as_secs_f64();
        for (i, graph_function) in functions.iter().enumerate() {
            graph_values[i].borrow_mut().push((elapsed, (graph_function.func)(elapsed * std::f64::consts::PI)));
        }
        drawing_area.queue_draw();
        glib::ControlFlow::Continue
    });
}

fn set_draw_func(start_time: &Rc<Instant>, graph_values: &Vec<Rc<RefCell<Vec<(f64, f64)>>>>, drawing_area: &DrawingArea, functions: &Rc<Vec<GraphFunction>>) {
    let start_time = Rc::clone(start_time);
    let graph_values = graph_values.clone();
    let functions = Rc::clone(functions);
    drawing_area.set_draw_func(move |_, cr, width, height| {
        render_graph(cr, width, height, start_time.clone(), &graph_values, &*functions, 40.0);
    });
}


fn render_graph(cr: &cairo::Context, width: i32, height: i32, start_time: Rc<Instant>, graph_values: &Vec<Rc<RefCell<Vec<(f64, f64)>>>>, functions: &[GraphFunction], window_size: f64) {
    let elapsed = start_time.elapsed().as_secs_f64();
    let backend = plotters_cairo::CairoBackend::new(cr, (width as u32, height as u32)).unwrap();
    let root = backend.into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Line Graph", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d((elapsed-window_size).max(0.0)..elapsed, -1.2..1.2)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    for (i, graph_function) in functions.iter().enumerate() {
        chart
            .draw_series(LineSeries::new(
                graph_values[i].borrow().iter().cloned(),
                &graph_function.color,
            ))
            .unwrap()
            .label(&graph_function.label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &graph_function.color));
    }

    chart.configure_series_labels().draw().unwrap();
}
