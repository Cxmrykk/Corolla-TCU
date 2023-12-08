use gtk4::cairo;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
use plotters::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

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

    let start_time = Rc::new(Instant::now());
    let current_value = Rc::new(RefCell::new(0.0));

    {
        let start_time = start_time.clone();
        let current_value = current_value.clone();
        let drawing_area = drawing_area.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(25), move || {
            let elapsed = start_time.elapsed().as_secs_f64();
            *current_value.borrow_mut() = (elapsed as f64).sin().abs();
            drawing_area.queue_draw();
            glib::ControlFlow::Continue
        });
    }

    drawing_area.set_draw_func(move |_, cr, width, height| {
        render_graph(cr, width, height, start_time.clone(), current_value.clone());
    });

    window
}

fn render_graph(cr: &cairo::Context, width: i32, height: i32, start_time: Rc<Instant>, current_value: Rc<RefCell<f64>>) {
    let elapsed = start_time.elapsed().as_secs_f64();
    let backend = plotters_cairo::CairoBackend::new(cr, (width as u32, height as u32)).unwrap();
    let root = backend.into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Line Graph", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0.0..elapsed, 0.0..1.2)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            (0..(elapsed as i32 * 100)).map(|x| x as f64 / 100.0).map(|x| (x, if x < elapsed { x.sin() } else { *current_value.borrow() })),
            &RED,
        ))
        .unwrap()
        .label("y = value")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels().draw().unwrap();
}