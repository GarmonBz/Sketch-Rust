//! Shared headless end-to-end test harness (`iced_test::Emulator`).

use crate::{program, App, Message};

use iced_test::emulator::{self, Emulator, Mode};
use iced_test::futures::futures::channel::mpsc;
use iced_test::futures::futures::executor;
use iced_test::futures::futures::StreamExt;
use iced_test::program::Program;
use iced_test::Instruction;

use std::path::PathBuf;

pub(crate) const VIEWPORT: iced::Size = iced::Size::new(1400.0, 860.0);

pub(crate) struct Harness<P: Program + 'static> {
    pub(crate) program: P,
    pub(crate) emulator: Emulator<P>,
    pub(crate) events: mpsc::Receiver<emulator::Event<P>>,
}

impl<P: Program<Theme = iced::Theme> + 'static> Harness<P> {
    pub(crate) fn boot_at_size(program: P, viewport: iced::Size) -> Self {
        crate::ui::fonts::preload_sync();
        let (sender, events) = mpsc::channel(64);
        // Immediate: never wait on tasks — the boot tasks query a real
        // window/monitor that does not exist headlessly.
        let emulator = Emulator::new(sender, &program, Mode::Immediate, viewport);
        let mut harness = Self {
            program,
            emulator,
            events,
        };
        harness.settle().expect("emulator boots");
        harness
    }

    /// Performs emitted actions until the emulator reports Ready (Ok) or a
    /// failed instruction (Err with that instruction).
    pub(crate) fn settle(&mut self) -> Result<(), String> {
        loop {
            let event = executor::block_on(self.events.next())
                .expect("emulator runtime should not stop on its own");
            match event {
                emulator::Event::Action(action) => self.emulator.perform(&self.program, action),
                emulator::Event::Failed(instruction) => return Err(instruction.to_string()),
                emulator::Event::Ready => return Ok(()),
            }
        }
    }

    pub(crate) fn step(&mut self, line: &str) -> Result<(), String> {
        let instruction =
            Instruction::parse(line).map_err(|e| format!("bad instruction {line:?}: {e:?}"))?;
        self.emulator.run(&self.program, instruction);
        self.settle()
    }

    pub(crate) fn script(&mut self, lines: &[&str]) {
        for line in lines {
            if let Err(failed) = self.step(line) {
                panic!("instruction failed: {failed} (script step {line:?})");
            }
            // Widget-operation tasks (a dialog's focus + select-all) run on
            // the executor and can emit just AFTER Ready; let them land
            // before the next instruction, as a real event loop would.
            self.wait(15);
        }
    }

    /// Deliver a message directly — only for input the emulator cannot
    /// script (mouse wheel, middle button, a held Space); the canvas side of
    /// those is covered by sketch_canvas's event-mapping tests.
    pub(crate) fn send(&mut self, message: P::Message) {
        // `Emulator::update` never reports Ready (only `run` does), so drain
        // whatever its task emits instead of settling.
        self.emulator.update(&self.program, message);
        self.wait(15);
    }

    /// True if `text` is currently on screen.
    pub(crate) fn shows(&mut self, text: &str) -> bool {
        self.step(&format!("expect {text:?}")).is_ok()
    }

    /// Let real time pass, then perform whatever the program's tasks emitted
    /// meanwhile (e.g. the toast timer).
    pub(crate) fn wait(&mut self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        while let Ok(event) = self.events.try_recv() {
            if let emulator::Event::Action(action) = event {
                self.emulator.perform(&self.program, action);
            }
        }
    }

    /// Writes a headless screenshot to `target/<subdir>/<name>.png`.
    pub(crate) fn screenshot_in(&mut self, subdir: &str, name: &str) -> PathBuf {
        let shot = self
            .emulator
            .screenshot(&self.program, &iced::Theme::Dark, 1.0);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("target")
            .join(subdir);
        std::fs::create_dir_all(&dir).expect("create screenshot dir");
        let path = dir.join(format!("{name}.png"));
        let file = std::fs::File::create(&path).expect("create screenshot file");
        let mut encoder = png::Encoder::new(
            std::io::BufWriter::new(file),
            shot.size.width,
            shot.size.height,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut w| w.write_image_data(&shot.rgba))
            .expect("write screenshot png");
        path
    }
}

pub(crate) fn harness_next_at(
    viewport: iced::Size,
) -> Harness<impl Program<State = App, Message = Message, Theme = iced::Theme> + 'static> {
    let mut h = Harness::boot_at_size(program(1.0), viewport);
    h.send(Message::WindowResized(viewport));
    h
}

pub(crate) fn state<P: Program<State = App> + 'static>(h: Harness<P>) -> App {
    h.emulator.into_state().0
}

/// One scroll area on screen: its id (if it has one), its own bounds and
/// the bounds of its content.
#[derive(Debug, Clone)]
pub(crate) struct ScrollArea {
    pub id: Option<String>,
    pub bounds: iced::Rectangle,
    pub content: iced::Rectangle,
}

impl ScrollArea {
    /// The content is taller or wider than the area, so it would scroll.
    pub(crate) fn overflows(&self) -> bool {
        self.content.height > self.bounds.height + 1.0
            || self.content.width > self.bounds.width + 1.0
    }
}

/// Lays out `app`'s whole view at `size` and lists every scroll area in it,
/// with the size of its content. A dialog "fits" when none of the scroll
/// areas around its form overflow.
pub(crate) fn scroll_areas(app: &App, size: iced::Size) -> Vec<ScrollArea> {
    use iced_test::selector::{Candidate, Selector};
    use std::sync::{Arc, Mutex};

    struct Collect(Arc<Mutex<Vec<ScrollArea>>>);
    impl Selector for Collect {
        type Output = ();
        fn select(&mut self, candidate: Candidate<'_>) -> Option<()> {
            if let Candidate::Scrollable {
                id,
                bounds,
                content_bounds,
                ..
            } = candidate
            {
                self.0.lock().expect("collector").push(ScrollArea {
                    id: id.map(|id| format!("{id:?}")),
                    bounds,
                    content: content_bounds,
                });
            }
            None
        }
        fn description(&self) -> String {
            "every scroll area".to_owned()
        }
    }

    crate::ui::fonts::preload_sync();
    let found = Arc::new(Mutex::new(Vec::new()));
    // A CPU (tiny-skia) renderer, asked for by name. `Simulator` would pick
    // the GPU backend first and create a new device on every call; after a
    // few dozen calls that hung the test run.
    let mut renderer = iced_test::futures::futures::executor::block_on(
        <iced::Renderer as iced_test::core::renderer::Headless>::new(
            crate::theme::SEGOE,
            iced::Pixels(16.0),
            Some("tiny-skia"),
        ),
    )
    .expect("tiny-skia headless renderer");
    let mut ui = iced_test::runtime::UserInterface::build(
        app.view(),
        size,
        iced_test::runtime::user_interface::Cache::default(),
        &mut renderer,
    );
    let mut operation = Collect(found.clone()).find();
    ui.operate(
        &renderer,
        &mut iced_test::core::widget::operation::black_box(&mut operation),
    );
    let out = found.lock().expect("collector").clone();
    out
}
