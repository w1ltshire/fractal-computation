#[cfg(target_arch = "wasm32")]
use wasm_thread as thread;

/// The [`egui`] app implementation.
pub mod app;
/// Implementation of [`tiles::Tiles`]
pub mod tiles;
/// Threaded computation
pub mod threads;

// if targeting desktop or smth
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use std::default::Default;

    env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "fractal-computation",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    console_log::init_with_level(log::Level::Info).unwrap();
    console_error_panic_hook::set_once();

    log::info!("trans rights are human rights 🏳️‍⚧️");
    log::info!("available parallelism {:?}", thread::available_parallelism());

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("egui-canvas")
            .expect("Failed to find egui-canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("egui-canvas was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
