//! Opens the original web player in a window, logging the API calls it makes and saving the
//! responses named in `record.js`, for studying the original and refreshing the saved session.

mod saving;

use max_api::session::{USER_AGENT, WEB_ORIGIN};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

type Failure = Box<dyn std::error::Error + Send + Sync>;

const WINDOW_SIZE: LogicalSize<f64> = LogicalSize::new(1280.0, 720.0);
const EXTRA_SCRIPT_VARIABLE: &str = "MAX_EXTRA_SCRIPT";
const CAPTURE_PREFIX: &str = "{\"capture\"";
const API_CALL_PREFIX: &str = "{\"api\"";

enum UserEvent {
    HomeCaptured,
}

fn without_query(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or(url)
}

fn main() -> wry::Result<()> {
    let url = std::env::args().nth(1).unwrap_or_else(|| WEB_ORIGIN.to_string());
    let extra_script = std::env::var(EXTRA_SCRIPT_VARIABLE).ok().and_then(|path| std::fs::read_to_string(path).ok());

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().with_title("web-capture").with_inner_size(WINDOW_SIZE).build(&event_loop).expect("create window");
    let web_view = WebViewBuilder::new()
        .with_initialization_script(extra_script.unwrap_or_default())
        .with_url(&url)
        .with_user_agent(USER_AGENT)
        .with_devtools(true)
        .with_initialization_script(include_str!("probe.js"))
        .with_initialization_script(include_str!("record.js"))
        .with_ipc_handler(move |request| {
            let message = request.body();
            if !message.starts_with(CAPTURE_PREFIX) {
                return println!("{} {message}", if message.starts_with(API_CALL_PREFIX) { "API" } else { "PROBE" });
            }
            match saving::save_capture(message) {
                Ok(name) => {
                    println!("CAPTURE {name}");
                    if name == saving::HOME_CAPTURE {
                        let _ = proxy.send_event(UserEvent::HomeCaptured);
                    }
                }
                Err(error) => eprintln!("capture failed: {error}"),
            }
        })
        .with_navigation_handler(|url| {
            println!("NAV {}", without_query(&url));
            true
        })
        .build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            // Cookies are read here and not in the message handler, which has no web view to ask.
            Event::UserEvent(UserEvent::HomeCaptured) => match saving::save_session(&web_view) {
                Ok(()) => println!("SESSION saved"),
                Err(error) => eprintln!("session save failed: {error}"),
            },
            _ => {}
        }
    });
}
