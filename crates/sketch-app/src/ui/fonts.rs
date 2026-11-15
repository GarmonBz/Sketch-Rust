//! Bundled Barlow & Barlow Semi Condensed fonts (SIL OFL 1.1).

#[cfg(test)]
use std::borrow::Cow;

use iced::font::{Family, Weight};
use iced::{Font, Task};

use crate::Message;

pub const BARLOW_REGULAR_BYTES: &[u8] =
    include_bytes!("../../../../assets/fonts/Barlow-Regular.ttf");
pub const BARLOW_MEDIUM_BYTES: &[u8] =
    include_bytes!("../../../../assets/fonts/Barlow-Medium.ttf");
pub const BARLOW_SEMIBOLD_BYTES: &[u8] =
    include_bytes!("../../../../assets/fonts/Barlow-SemiBold.ttf");
pub const BSC_MEDIUM_BYTES: &[u8] =
    include_bytes!("../../../../assets/fonts/BarlowSemiCondensed-Medium.ttf");
pub const BSC_SEMIBOLD_BYTES: &[u8] =
    include_bytes!("../../../../assets/fonts/BarlowSemiCondensed-SemiBold.ttf");

pub const ALL_FONTS: [&[u8]; 5] = [
    BARLOW_REGULAR_BYTES,
    BARLOW_MEDIUM_BYTES,
    BARLOW_SEMIBOLD_BYTES,
    BSC_MEDIUM_BYTES,
    BSC_SEMIBOLD_BYTES,
];

pub const BARLOW: Font = Font {
    family: Family::Name("Barlow"),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

pub const BARLOW_MEDIUM: Font = Font {
    family: Family::Name("Barlow"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

pub const BARLOW_SEMIBOLD: Font = Font {
    family: Family::Name("Barlow"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};

pub const BSC_MEDIUM: Font = Font {
    family: Family::Name("Barlow Semi Condensed"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};

pub const BSC_SEMIBOLD: Font = Font {
    family: Family::Name("Barlow Semi Condensed"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};

/// Synchronously registers all bundled Barlow fonts in Iced's global
/// `FontSystem` (used by headless `iced_test::Emulator` harnesses because
/// `Emulator` leaves `Action::LoadFont` unimplemented).
#[cfg(test)]
pub fn preload_sync() {
    if let Ok(mut system) = iced::advanced::graphics::text::font_system().write() {
        for bytes in ALL_FONTS {
            system.load_font(Cow::Borrowed(bytes));
        }
    }
}

/// Loads all bundled Barlow fonts into Iced's font database at boot.
pub fn load_tasks() -> Task<Message> {
    Task::batch([
        iced::font::load(BARLOW_REGULAR_BYTES).map(|_| Message::Noop),
        iced::font::load(BARLOW_MEDIUM_BYTES).map(|_| Message::Noop),
        iced::font::load(BARLOW_SEMIBOLD_BYTES).map(|_| Message::Noop),
        iced::font::load(BSC_MEDIUM_BYTES).map(|_| Message::Noop),
        iced::font::load(BSC_SEMIBOLD_BYTES).map(|_| Message::Noop),
    ])
}
