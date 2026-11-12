//! `EventTap` — a transparent root wrapper that reports pointer presses (with
//! their position) and the Escape key, then passes every event on to its
//! content unchanged.
//!
//! It replaces a global `event::listen_with` subscription for the File
//! backstage's close rules (refs/ui-manager.js: a document-level `pointerdown`
//! outside `#file-menu` and the File tab closes it; Escape closes it; the
//! press still reaches its target). Routing these through the widget tree
//! means the real window and the headless `iced_test` emulator — which only
//! delivers events to widgets, never to subscriptions — behave identically.

use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::keyboard;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

pub struct EventTap<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    /// A right press (the original's `contextmenu` handlers).
    on_right_press: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    /// The last cursor position seen: iced reports a RIGHT press with an
    /// unavailable cursor (both here and in iced_test), so the right handler
    /// falls back to where the pointer last was.
    last_cursor: Option<Point>,
    on_escape: Option<Message>,
    on_move: Option<Box<dyn Fn(Point, f32) -> Message + 'a>>,
    on_release: Option<Message>,
    on_key: Option<KeyHandler<'a, Message>>,
    on_key_up: Option<KeyHandler<'a, Message>>,
    keyboard_action: Option<(iced::advanced::widget::Id, Message)>,
    capture_key: Option<KeyHandler<'a, Message>>,
    focused_key: Option<KeyHandler<'a, Message>>,
}

#[derive(Default)]
struct KeyboardFocus(bool);
impl iced::advanced::widget::operation::Focusable for KeyboardFocus {
    fn is_focused(&self) -> bool {
        self.0
    }
    fn focus(&mut self) {
        self.0 = true;
    }
    fn unfocus(&mut self) {
        self.0 = false;
    }
}

/// A shortcut handler: (key, modifiers) -> message, or None to ignore.
type KeyHandler<'a, Message> =
    Box<dyn Fn(&keyboard::Key, keyboard::Modifiers) -> Option<Message> + 'a>;

impl<'a, Message, Theme, Renderer> EventTap<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            on_right_press: None,
            last_cursor: None,
            on_escape: None,
            on_move: None,
            on_release: None,
            on_key: None,
            on_key_up: None,
            keyboard_action: None,
            capture_key: None,
            focused_key: None,
        }
    }

    /// Any mouse button press anywhere inside the tap, with its position.
    pub fn on_press(mut self, f: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_press = Some(Box::new(f));
        self
    }

    /// A RIGHT press (the original's `contextmenu` handlers).
    pub fn on_right_press(mut self, f: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_right_press = Some(Box::new(f));
        self
    }

    /// The Escape key.
    pub fn on_escape(mut self, message: Message) -> Self {
        self.on_escape = Some(message);
        self
    }

    /// Every cursor move (position + the tap's width, i.e. the viewport's) —
    /// pointer capture for drags that leave their handle.
    pub fn on_move(mut self, f: impl Fn(Point, f32) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(f));
        self
    }

    /// Any mouse button release.
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    /// Keyboard shortcuts: offered AFTER the content, and only for key
    /// presses no widget captured (a focused text input keeps its typing —
    /// InputController.handleKeyDown ignores keys aimed at inputs).
    pub fn on_key(
        mut self,
        f: impl Fn(&keyboard::Key, keyboard::Modifiers) -> Option<Message> + 'a,
    ) -> Self {
        self.on_key = Some(Box::new(f));
        self
    }

    /// Key releases no widget captured (InputController.handleKeyUp).
    pub fn on_key_up(
        mut self,
        f: impl Fn(&keyboard::Key, keyboard::Modifiers) -> Option<Message> + 'a,
    ) -> Self {
        self.on_key_up = Some(Box::new(f));
        self
    }

    /// Focusable wrapper for canvas-palette tiles and workflow buttons.
    pub fn keyboard_action(
        mut self,
        id: impl Into<iced::advanced::widget::Id>,
        message: Message,
    ) -> Self {
        self.keyboard_action = Some((id.into(), message));
        self
    }

    /// Modal navigation must be intercepted before focused fields consume it.
    pub fn capture_key(
        mut self,
        f: impl Fn(&keyboard::Key, keyboard::Modifiers) -> Option<Message> + 'a,
    ) -> Self {
        self.capture_key = Some(Box::new(f));
        self
    }

    pub fn focused_key(
        mut self,
        f: impl Fn(&keyboard::Key, keyboard::Modifiers) -> Option<Message> + 'a,
    ) -> Self {
        self.focused_key = Some(Box::new(f));
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for EventTap<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<KeyboardFocus>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(KeyboardFocus::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if let Some((id, _)) = &self.keyboard_action {
            operation.focusable(
                Some(id),
                layout.bounds(),
                tree.state.downcast_mut::<KeyboardFocus>(),
            );
        }
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
            if let Some(message) = self.capture_key.as_ref().and_then(|f| f(key, *modifiers)) {
                shell.publish(message);
                shell.capture_event();
                return;
            }
        }
        if let Some((_, message)) = &self.keyboard_action {
            let state = tree.state.downcast_mut::<KeyboardFocus>();
            if matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            ) {
                state.0 = cursor.is_over(layout.bounds());
            }
            if state.0 {
                if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                    if let Some(message) =
                        self.focused_key.as_ref().and_then(|f| f(key, *modifiers))
                    {
                        shell.publish(message);
                        shell.capture_event();
                        return;
                    }
                }
            }
            let space_or_enter = match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    matches!(
                        key,
                        keyboard::Key::Named(
                            keyboard::key::Named::Enter | keyboard::key::Named::Space
                        )
                    ) || matches!(key,keyboard::Key::Character(s) if s.as_str()==" ")
                }
                _ => false,
            };
            if state.0 && space_or_enter {
                shell.publish(message.clone());
                shell.capture_event();
                return;
            }
            if state.0
                && matches!(event,Event::Keyboard(keyboard::Event::KeyReleased{key,..}) if matches!(key,keyboard::Key::Named(keyboard::key::Named::Space)) || matches!(key,keyboard::Key::Character(s) if s.as_str()==" "))
            {
                shell.capture_event();
                return;
            }
        }
        // Observe first (like a capture-phase document listener), never
        // capture: the content still receives the event.
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                if let Some(p) = cursor.position_over(layout.bounds()) {
                    self.last_cursor = Some(p);
                }
                if let (Some(f), Some(p)) = (&self.on_press, cursor.position_over(layout.bounds()))
                {
                    shell.publish(f(p));
                }
                if *button == mouse::Button::Right {
                    let at = cursor.position_over(layout.bounds()).or(self.last_cursor);
                    if let (Some(f), Some(p)) = (&self.on_right_press, at) {
                        shell.publish(f(p));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                self.last_cursor = Some(*position);
                if let Some(f) = &self.on_move {
                    shell.publish(f(*position, layout.bounds().width));
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(_)) => {
                if let Some(m) = &self.on_release {
                    shell.publish(m.clone());
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                if let Some(m) = &self.on_escape {
                    shell.publish(m.clone());
                }
            }
            _ => {}
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if let (Some(f), Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })) =
            (&self.on_key, event)
        {
            if !shell.is_event_captured() {
                if let Some(m) = f(key, *modifiers) {
                    shell.publish(m);
                }
            }
        }
        if let (Some(f), Event::Keyboard(keyboard::Event::KeyReleased { key, modifiers, .. })) =
            (&self.on_key_up, event)
        {
            if !shell.is_event_captured() {
                if let Some(m) = f(key, *modifiers) {
                    shell.publish(m);
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if self.keyboard_action.is_some() && tree.state.downcast_ref::<KeyboardFocus>().0 {
            renderer.fill_quad(
                iced::advanced::renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border {
                        color: iced::Color::from_rgb8(99, 179, 237),
                        width: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<EventTap<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(tap: EventTap<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tap)
    }
}
