/// A logo's width and height scaled to `height`, read from a PNG's header or an SVG's
/// `width` and `height`, else its `viewBox`; None for any other file.
pub(crate) fn measure_logo(bytes: &[u8], height: u32) -> Option<(u32, u32)> {
    let (width, own) = read_png_size(bytes).or_else(|| read_svg_size(bytes))?;
    if width <= 0.0 || own <= 0.0 {
        return None;
    }
    let scaled = (f64::from(height) * width / own).round();
    Some((scaled as u32, height))
}

fn read_png_size(bytes: &[u8]) -> Option<(f64, f64)> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }
    let read = |at: usize| -> Option<f64> {
        let word: [u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
        Some(f64::from(u32::from_be_bytes(word)))
    };
    Some((read(16)?, read(20)?))
}

fn read_svg_size(bytes: &[u8]) -> Option<(f64, f64)> {
    let text = std::str::from_utf8(bytes).ok()?;
    let start = text.find("<svg")?;
    let tag = &text[start..start + text[start..].find('>')?];
    let number = |value: &str| value.trim().trim_end_matches("px").parse::<f64>().ok();
    let width = read_attribute(tag, "width").and_then(number);
    let height = read_attribute(tag, "height").and_then(number);
    if let (Some(width), Some(height)) = (width, height) {
        return Some((width, height));
    }
    let view: Vec<f64> = (read_attribute(tag, "viewBox")?)
        .split(|char: char| char.is_whitespace() || char == ',')
        .filter(|part| !part.is_empty())
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    match view.as_slice() {
        [_, _, width, height] => Some((*width, *height)),
        _ => None,
    }
}

// The value of ` name="…"` in a start tag; a percentage or other unit is no size
fn read_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let rest = tag.split(&format!(" {name}=")).nth(1)?;
    let quote = rest
        .chars()
        .next()
        .filter(|char| *char == '"' || *char == '\'')?;
    let value = rest[1..].split(quote).next()?;
    (!value.contains('%') && !value.contains("em")).then_some(value)
}

#[cfg(test)]
#[path = "measure_unit.rs"]
mod unit;
