//! `.app-toast` — UIManager.notify(message): a fixed toast centered at the
//! bottom (`bottom: 54px`), shown for 3.2s; a new message replaces the
//! current one and restarts the timer (`clearTimeout(this.toastTimer)`).
//! Measured from the original: 16px Segoe UI, padding 9px 14px, 1px
//! `--border`, radius 4, `--bg-panel`, shadow `0 5px 18px rgba(0,0,0,.35)`.

use iced::widget::{column, container, row, Space};
use iced::{Border, Color, Element, Length, Padding, Shadow, Task, Vector};

use crate::theme::{bg_panel, border, label, text_main, SEGOE};
use crate::Message;

/// `setTimeout(() => toast.classList.remove('show'), 3200)`.
pub const SHOW_MS: u64 = 3200;

#[derive(Debug, Default)]
pub struct Toasts {
    /// The message on display and its generation.
    pub current: Option<(u64, String)>,
    seq: u64,
}

impl Toasts {
    /// notify(message): show it and (re)start the 3.2s timer.
    pub fn show(&mut self, message: String) -> Task<Message> {
        self.seq += 1;
        let id = self.seq;
        self.current = Some((id, message));
        Task::perform(
            async move {
                let (tx, rx) = iced::futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(SHOW_MS));
                    let _ = tx.send(());
                });
                let _ = rx.await;
                id
            },
            Message::ToastExpired,
        )
    }

    /// The timer of generation `id` fired; stale timers are ignored.
    pub fn expire(&mut self, id: u64) {
        if self.current.as_ref().is_some_and(|(c, _)| *c == id) {
            self.current = None;
        }
    }

    pub fn text(&self) -> Option<&str> {
        self.current.as_ref().map(|(_, t)| t.as_str())
    }
}

/// The overlay layer (pointer-events: none — it holds no interactive widget).
pub fn layer<'a>(message: &str) -> Element<'a, Message> {
    let toast = container(label(message, 16.0, SEGOE, text_main(), 21.0))
        .padding(Padding {
            // CSS: 1px border + 9px padding around a 21px line = 41px tall.
            // iced draws the border inside the padding, so 10 + 21 + 10
            // (the text then lands on the original's rows, measured).
            top: 10.0,
            bottom: 10.0,
            left: 14.0,
            right: 14.0,
        })
        .style(|_| container::Style {
            background: Some(bg_panel().into()),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: Vector::new(0.0, 5.0),
                blur_radius: 18.0,
            },
            ..container::Style::default()
        });
    column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            toast,
            Space::new().width(Length::Fill)
        ],
        Space::new().height(54),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
