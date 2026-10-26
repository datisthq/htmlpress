use crate::icons::ICONS;

/// The inline `<svg>` for a built-in icon, drawn in the current text color; None for a
/// name the set does not have.
pub(crate) fn render_icon(name: &str) -> Option<String> {
    let index = ICONS.binary_search_by(|(icon, _)| (*icon).cmp(name)).ok()?;
    let (_, markup) = ICONS[index];
    Some(format!(
        "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" \
         stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">{markup}</svg>"
    ))
}

/// Every built-in icon's name, for an error that names an unknown one.
pub(crate) fn list_icons() -> String {
    ICONS
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "render_unit.rs"]
mod unit;
