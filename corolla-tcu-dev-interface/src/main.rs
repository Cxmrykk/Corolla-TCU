use std::{
    io::Read,
    time::{Duration, Instant},
    sync::{Arc, Mutex},
    thread,
};

use gtk4::{MenuButton, gio, prelude::*};
use gtk4::{cairo, Application, ApplicationWindow, DrawingArea, Entry, Box as GtkBox};
use plotters::prelude::*;

const START_MARKER: u8 = 0x01; // SOH
const END_MARKER: u8 = 0x0D; // LINE FEED
const SERIAL_PORT: &str = "/dev/ttyACM0";
const BAUD_RATE: u32 = 115_200;

const WINDOW_SIZE_DEFAULT: f64 = 10.0;
const WINDOW_REFRESH: u64 = 1;

#[derive(Clone)]
struct GraphFunction {
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
    let window_size = Arc::new(Mutex::new(WINDOW_SIZE_DEFAULT));
    let graph_values = create_graph_values(&functions);
    let sized_values = sized_graph_values(&graph_values, &window_size);
    let start_time = Arc::new(Instant::now());

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    window.set_child(Some(&vbox));

    let gpio_states = Arc::new(Mutex::new(vec![0; 8]));

    spawn_serial_thread(Arc::clone(&gpio_states));

    let settings_action = gio::SimpleAction::new("settings", None);
    app.add_action(&settings_action);
    app.set_accels_for_action("app.settings", &["<primary>S"]);

    let window_size_clone = Arc::clone(&window_size);
    settings_action.connect_activate(move |_, _| {
        create_settings_window(Arc::clone(&window_size_clone));
    });

    let menu_model = gio::Menu::new();
    menu_model.append(Some("Settings"), Some("app.settings"));

    let menu_button = MenuButton::new();
    menu_button.set_menu_model(Some(&menu_model));
    menu_button.set_label("Settings");
    vbox.append(&menu_button);


    for i in 0..functions.len() {
        let drawing_area = create_drawing_area();
        vbox.append(&drawing_area);
        set_timeout(&start_time, &window_size, &graph_values[i], &sized_values[i], &drawing_area, Arc::clone(&gpio_states), i);
        set_draw_func(&start_time, &sized_values[i], &drawing_area, &functions[i], Arc::clone(&window_size));
    }

    window
}

fn create_settings_window(window_size: Arc<Mutex<f64>>) {
    let window = ApplicationWindow::builder()
        .title("Settings")
        .default_width(200)
        .default_height(100)
        .build();

    let vbox = GtkBox::new(gtk4::Orientation::Vertical, 0);
    window.set_child(Some(&vbox));

    let entry = Entry::new();
    entry.set_placeholder_text(Some("Enter window size"));
    vbox.append(&entry);

    let button = gtk4::Button::with_label("Set");
    button.connect_clicked(move |_| {
        if let Ok(value) = entry.text().parse::<f64>() {
            let mut window_size = window_size.lock().unwrap();
            *window_size = value;
        }
    });
    vbox.append(&button);

    window.show();
}

fn spawn_serial_thread(gpio_states: Arc<Mutex<Vec<u8>>>) {
    thread::spawn({
        let gpio_states = Arc::clone(&gpio_states);
        move || {
            let s = serialport::new(SERIAL_PORT, BAUD_RATE)
                .timeout(Duration::from_millis(1))
                .open()
                .unwrap();

            let mut serial_buf: Vec<u8> = vec![0; 10];
            let mut serial_port = s;

            loop {
                match serial_port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        if t == 10 {
                            if serial_buf[0] == START_MARKER && serial_buf[9] == END_MARKER {
                                for i in 1..=8 {
                                    let mut gpio_states = gpio_states.lock().unwrap();
                                    gpio_states[i-1] = serial_buf[i];
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
    });
}

fn create_drawing_area() -> DrawingArea {
    let drawing_area = DrawingArea::new();
    drawing_area.set_vexpand(true);
    drawing_area.set_hexpand(true);
    drawing_area.set_size_request(-1, -1);  // Set a minimum height of 200
    drawing_area
}

fn create_functions() -> Vec<GraphFunction> {
    vec![
        GraphFunction { label: "Line Pressure VCC".to_string(), color: RED }, // GPIO40
        GraphFunction { label: "Shift Solenoid 1".to_string(), color: BLUE }, // GPIO41
        GraphFunction { label: "Shift Solenoid 2".to_string(), color: GREEN }, // GPIO42
        GraphFunction { label: "TC Lockup".to_string(), color: CYAN }, // GPIO43
        GraphFunction { label: "Line Pressure GND".to_string(), color: MAGENTA }, // GPIO44
        GraphFunction { label: "VSS".to_string(), color: YELLOW }, // GPIO45
        GraphFunction { label: "RPM".to_string(), color: BLACK }, // GPIO46

        // Add more functions for other GPIO states
    ]
}

fn create_graph_values(functions: &[GraphFunction]) -> Vec<Arc<Mutex<Vec<(f64, f64)>>>> {
    functions.iter().map(|_| Arc::new(Mutex::new(Vec::new()))).collect()
}

fn sized_graph_values(graph_values: &Vec<Arc<Mutex<Vec<(f64, f64)>>>>, window_size: &Arc<Mutex<f64>>) -> Vec<Arc<Mutex<Vec<(f64, f64)>>>> {
    let size = (*window_size.lock().unwrap() * 1000.0) * f64::from(WINDOW_REFRESH as u32);
    graph_values.iter().map(|rc_refcell_vec| {
        let vec = rc_refcell_vec.lock().unwrap();
        let len = vec.len();
        let start = if len > size as usize { len - size as usize } else { 0 };
        Arc::new(Mutex::new(vec[start..].to_vec()))
    }).collect()
}

fn set_timeout(
    start_time: &Arc<Instant>,
    window_size: &Arc<Mutex<f64>>,
    graph_value: &Arc<Mutex<Vec<(f64, f64)>>>,
    sized_value: &Arc<Mutex<Vec<(f64, f64)>>>,
    drawing_area: &DrawingArea,
    gpio_states: Arc<Mutex<Vec<u8>>>,
    index: usize,
) {
    let window_size = Arc::clone(window_size);
    let start_time = Arc::clone(start_time);
    let graph_value = Arc::clone(graph_value);
    let sized_value = Arc::clone(sized_value);
    let drawing_area = drawing_area.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(WINDOW_REFRESH), move || {
        let elapsed = start_time.elapsed().as_secs_f64();
        let gpio_states = gpio_states.lock().unwrap();
        let state = (gpio_states[index] >> index) & 0x01;
        
        // update graph values
        {
            let mut graph_value = graph_value.lock().unwrap();
            graph_value.push((elapsed, state as f64));
        }
        
        // update sized values
        {
            let size = (*window_size.lock().unwrap() * 1000.0) * f64::from(WINDOW_REFRESH as u32);
            let graph_value = graph_value.lock().unwrap();
            let len = graph_value.len();
            let start = if len > size as usize { len - size as usize } else { 0 };
            let mut sized_value = sized_value.lock().unwrap();
            *sized_value = graph_value[start..].to_vec();
        }

        drawing_area.queue_draw();
        glib::ControlFlow::Continue
    });
}

fn set_draw_func(
    start_time: &Arc<Instant>,
    sized_value: &Arc<Mutex<Vec<(f64, f64)>>>,
    drawing_area: &DrawingArea,
    function: &GraphFunction,
    window_size: Arc<Mutex<f64>>,
) {
    let start_time = Arc::clone(start_time);
    let sized_value = Arc::clone(sized_value);
    let function = function.clone();
    let window_size = Arc::clone(&window_size);

    drawing_area.set_draw_func(move |_, cr, width, height| {
        if width > 0 && height > 0 {
            let window_size = window_size.lock().unwrap();
            render_graph(cr, width, height, Arc::clone(&start_time), &*sized_value.lock().unwrap(), &function, *window_size);
        }
    });
}

fn render_graph(
    cr: &cairo::Context,
    width: i32,
    height: i32,
    start_time: Arc<Instant>,
    graph_value: &[(f64, f64)],
    function: &GraphFunction,
    window_size: f64,
) {
    let elapsed = start_time.elapsed().as_secs_f64();
    let backend = plotters_cairo::CairoBackend::new(cr, (width as u32, height as u32)).unwrap();
    let root = backend.into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption(&function.label, ("sans-serif", 16).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(0)
        .build_cartesian_2d((elapsed - window_size).max(0.0)..elapsed, 0.0..1.0)
        .unwrap();

    chart.configure_mesh()
        .disable_y_mesh()
        .draw()
        .unwrap();

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
