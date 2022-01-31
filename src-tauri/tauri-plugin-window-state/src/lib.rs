use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Plugin, Result as PluginResult},
    AppHandle, Event, PhysicalPosition, Position, Runtime, Window, WindowEvent,
};

use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    sync::{Arc, Mutex},
};

const STATE_FILENAME: &str = ".window-state";

#[derive(Debug, Default, Deserialize, Serialize)]
struct WindowMetadata {
    x: i32,
    y: i32,
}

#[derive(Default)]
pub struct WindowState {
    cache: Arc<Mutex<HashMap<String, WindowMetadata>>>,
}

impl<R: Runtime> Plugin<R> for WindowState {
    fn name(&self) -> &'static str {
        "window-state"
    }

    fn initialize(&mut self, app: &AppHandle<R>, _config: serde_json::Value) -> PluginResult<()> {
        if let Some(app_dir) = app.path_resolver().app_dir() {
            let state_path = app_dir.join(STATE_FILENAME);
            if state_path.exists() {
                self.cache = Arc::new(Mutex::new(
                    tauri::api::file::read_binary(state_path)
                        .and_then(|state| bincode::deserialize(&state).map_err(Into::into))
                        .unwrap_or_default(),
                ));
            }
        }
        Ok(())
    }

    fn created(&mut self, window: Window<R>) {
        {
            let mut c = self.cache.lock().unwrap();
            if let Some(state) = c.get(window.label()) {
                window.set_position(Position::Physical(PhysicalPosition { x: state.x, y: state.y })).unwrap();
            } else {
                let PhysicalPosition { x, y } = window.outer_position().unwrap();
                c.insert(window.label().into(), WindowMetadata { x, y });
            }
        }

        let cache = self.cache.clone();
        let label = window.label().to_string();
        let window_clone = window.clone();

        window.on_window_event(move |e| match e {
            WindowEvent::Moved(position) => {
                let size = window_clone.inner_size().unwrap();
                // On some platforms, when a window gets minimized, it will report a
                // position with a huge negative value (-32000), so for now
                // don't save the position if the window is not visible on screen.
                // also subtract a 25px, just to ensure there is enough space to
                // be able to resize the window.
                // TODO: use `window.is_minimized()` once it is implemented
                if let Some(monitor) = window_clone.current_monitor().unwrap() {
                    let monitor_size = monitor.position();
                    if position.x > monitor_size.x - size.width as i32 - 25 && position.y > monitor_size.y - size.height as i32 - 25 {
                        let mut c = cache.lock().unwrap();
                        let state = c.get_mut(&label).unwrap();
                        state.x = position.x;
                        state.y = position.y;
                    };
                };
            }
            _ => {}
        });

        window.show().unwrap();
        window.set_focus().unwrap();
    }

    fn on_event(&mut self, app: &AppHandle<R>, event: &Event) {
        if let Event::Exit = event {
            if let Some(app_dir) = app.path_resolver().app_dir() {
                let state_path = app_dir.join(STATE_FILENAME);
                let state = self.cache.lock().unwrap();
                let _ = create_dir_all(&app_dir)
                    .map_err(tauri::api::Error::Io)
                    .and_then(|_| File::create(state_path).map_err(Into::into))
                    .and_then(|mut f| f.write_all(&bincode::serialize(&*state).map_err(tauri::api::Error::Bincode)?).map_err(Into::into));
            }
        }
    }
}
