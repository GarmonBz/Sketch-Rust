//! Fill style names shared by the area type editor.

pub const FILL_STYLES: [&str; 5] = ["solid", "none", "diagonal", "crosshatch", "dots"];
pub fn style_label(value: &str) -> &'static str {
    match value {
        "none" => "None",
        "diagonal" => "Diagonal",
        "crosshatch" => "Crosshatch",
        "dots" => "Dots",
        _ => "Solid",
    }
}
pub fn style_value(label: &str) -> &'static str {
    match label {
        "None" => "none",
        "Diagonal" => "diagonal",
        "Crosshatch" => "crosshatch",
        "Dots" => "dots",
        _ => "solid",
    }
}
