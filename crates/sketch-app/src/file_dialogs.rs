//! Native sketch-save picker, isolated so widget tests can choose a disposable
//! destination without opening a Windows dialog on the user's desktop.

use std::path::PathBuf;

/// File > Open: ask for an existing sketch file. Returns a future; in test
/// builds the TEST_PICKER boundary resolves it immediately (no real dialog).
pub async fn open_sketch() -> Option<PathBuf> {
    #[cfg(test)]
    {
        TEST_PICKER.with(|state| {
            let mut state = state.borrow_mut();
            state.open_requests += 1;
            state.open_destination.take()
        })
    }
    #[cfg(not(test))]
    crate::platform::pick_file_async(
        "Sketch".to_owned(),
        crate::SKETCH_OPEN_EXTENSIONS
            .iter()
            .map(|ext| (*ext).to_owned())
            .collect(),
    )
    .await
}

/// The image picker uses the same test boundary as sketch Open, so headless
/// layout checks never wait for an interactive Windows file dialog.
pub async fn open_image() -> Option<PathBuf> {
    #[cfg(test)]
    {
        TEST_PICKER.with(|state| {
            let mut state = state.borrow_mut();
            state.open_requests += 1;
            state.open_destination.take()
        })
    }
    #[cfg(not(test))]
    crate::platform::pick_file_async(
        "Images".to_owned(),
        vec!["png".to_owned(), "jpg".to_owned(), "jpeg".to_owned()],
    )
    .await
}

/// File > Save / Save As: ask where to save, offering `default_name`. In
/// test builds the TEST_PICKER boundary resolves immediately.
pub async fn save_sketch(default_name: String, preferred: String) -> Option<PathBuf> {
    #[cfg(test)]
    {
        let _ = preferred;
        TEST_PICKER.with(|state| {
            let mut state = state.borrow_mut();
            state.requests.push(default_name);
            state.destination.take()
        })
    }
    #[cfg(not(test))]
    {
        let mut filters = vec![
            ("Sketch", "skx"),
            ("Sketchie JSON", "sketchie.json"),
            ("Sketchie Store", "sketchie"),
            ("Sketch JSON", "json"),
        ];
        if matches!(preferred.as_str(), "sketchy.json" | "sketchy") {
            filters.extend([
                ("Sketchy JSON (legacy)", "sketchy.json"),
                ("Sketchy Store (legacy)", "sketchy"),
            ]);
        }
        let ordered: Vec<(String, String)> = filters
            .iter()
            .filter(|(_, ext)| preferred == *ext)
            .chain(filters.iter().filter(|(_, ext)| preferred != *ext))
            .map(|(name, ext)| ((*name).to_owned(), (*ext).to_owned()))
            .collect();
        crate::platform::save_file_as_async(default_name, ordered).await
    }
}

#[cfg(test)]
#[derive(Default)]
struct TestPicker {
    destination: Option<PathBuf>,
    requests: Vec<String>,
    open_destination: Option<PathBuf>,
    open_requests: usize,
}

#[cfg(test)]
thread_local! {
    static TEST_PICKER: std::cell::RefCell<TestPicker> = Default::default();
}

#[cfg(test)]
pub fn with_save_path<R>(
    destination: Option<PathBuf>,
    run: impl FnOnce() -> R,
) -> (R, Vec<String>) {
    let (result, picker) = with_picker(
        TestPicker {
            destination,
            ..Default::default()
        },
        run,
    );
    (result, picker.requests)
}

#[cfg(test)]
pub fn with_open_path<R>(destination: Option<PathBuf>, run: impl FnOnce() -> R) -> (R, usize) {
    let (result, picker) = with_picker(
        TestPicker {
            open_destination: destination,
            ..Default::default()
        },
        run,
    );
    (result, picker.open_requests)
}

#[cfg(test)]
fn with_picker<R>(picker: TestPicker, run: impl FnOnce() -> R) -> (R, TestPicker) {
    struct Restore(Option<TestPicker>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_PICKER.with(|state| *state.borrow_mut() = self.0.take().unwrap());
        }
    }
    let _restore = Restore(Some(TEST_PICKER.with(|state| state.replace(picker))));
    let result = run();
    let picker = TEST_PICKER.with(|state| state.take());
    (result, picker)
}

/// Runs `task` for a test, delivering each message it outputs back into
/// the app. The stream is polled directly on THIS thread (the TEST_PICKER
/// boundary is a thread-local); iced tasks open with an immediate `yield`,
/// so a Pending poll that woke the waker synchronously is polled again,
/// while a task that is really waiting (a toast timer takes
/// `toast::SHOW_MS`, 3.2 s) is dropped — exactly like the follow-up tasks
/// the old synchronous tests discarded. Call this inside the
/// `with_save_path` / `with_open_path` closure so the picker boundary is
/// installed while the dialog future is polled.
#[cfg(test)]
pub fn drive(app: &mut crate::App, task: iced::Task<crate::Message>) {
    use iced::futures::StreamExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    struct Wakes(AtomicUsize);
    impl Wake for Wakes {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    let Some(mut stream) = iced_runtime::task::into_stream(task) else {
        return;
    };
    let wakes = Arc::new(Wakes(AtomicUsize::new(0)));
    let waker = Waker::from(wakes.clone());
    let mut cx = Context::from_waker(&waker);
    loop {
        let before = wakes.0.load(Ordering::Relaxed);
        match stream.poll_next_unpin(&mut cx) {
            Poll::Ready(Some(iced_runtime::Action::Output(message))) => {
                let _ = app.update(message);
            }
            Poll::Ready(Some(_)) => {}
            Poll::Ready(None) => break,
            Poll::Pending => {
                // Only keep polling while something woke us (iced's opening
                // yield); a task that is truly waiting is dropped.
                if wakes.0.load(Ordering::Relaxed) == before {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_picker_cancels_and_loads_without_a_native_dialog() {
        let mut app = crate::App::boot(1.).0;
        let (_, requests) = with_open_path(None, || {
            let task = app.update(crate::Message::GeoLoadMap);
            drive(&mut app, task);
        });
        assert_eq!(requests, 1);
        assert!(!app.bg_image.loaded());
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/app-icon.png");
        let (_, requests) = with_open_path(Some(fixture), || {
            let task = app.update(crate::Message::PhotoLoad);
            drive(&mut app, task);
        });
        assert_eq!(requests, 1);
        assert!(app.bg_image.loaded());
        assert!(app.bg_image.width > 0 && app.bg_image.height > 0);
    }
}
