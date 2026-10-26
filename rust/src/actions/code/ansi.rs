const STANDARD: [&str; 8] = [
    "#4f5b66", "#bf616a", "#a3be8c", "#ebcb8b", "#8fa1b3", "#b48ead", "#96b5b4",
    "#c0c5ce",
];
const BRIGHT: [&str; 8] = [
    "#65737e", "#bf616a", "#a3be8c", "#ebcb8b", "#8fa1b3", "#b48ead", "#96b5b4",
    "#eff1f5",
];

#[derive(Default)]
struct State {
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    fg: Option<String>,
    bg: Option<String>,
}

/// Render terminal output's SGR escapes (`ESC[…m`) as styled spans, as Livemark's
/// `ansi` code blocks do; any other escape is dropped. `input` is plain text.
pub(crate) fn render_ansi(input: &str) -> String {
    let mut state = State::default();
    let mut html = String::new();
    let mut rest = input;
    while let Some(start) = rest.find('\u{1b}') {
        push_text(&mut html, &rest[..start], &state);
        let sequence = &rest[start + 1..];
        let Some(params) = sequence.strip_prefix('[') else {
            rest = sequence;
            continue;
        };
        let end = params.find(|c: char| !c.is_ascii_digit() && c != ';');
        let Some(end) = end else {
            rest = "";
            break;
        };
        if params[end..].starts_with('m') {
            apply_params(&mut state, &params[..end]);
        }
        rest = &params[end + 1..];
    }
    push_text(&mut html, rest, &state);
    html
}

fn push_text(html: &mut String, text: &str, state: &State) {
    if text.is_empty() {
        return;
    }
    let text = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    match create_style(state) {
        Some(style) => html.push_str(&format!("<span style=\"{style}\">{text}</span>")),
        None => html.push_str(&text),
    }
}

fn create_style(state: &State) -> Option<String> {
    let mut parts = vec![];
    if let Some(fg) = &state.fg {
        parts.push(format!("color:{fg}"));
    }
    if let Some(bg) = &state.bg {
        parts.push(format!("background-color:{bg}"));
    }
    let flags = [
        (state.bold, "font-weight:bold"),
        (state.dim, "opacity:0.7"),
        (state.italic, "font-style:italic"),
        (state.underline, "text-decoration:underline"),
        (state.strikethrough, "text-decoration:line-through"),
    ];
    parts.extend(
        flags
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, css)| css.to_string()),
    );
    (!parts.is_empty()).then(|| parts.join(";"))
}

fn apply_params(state: &mut State, text: &str) {
    let params: Vec<u32> = match text.is_empty() {
        true => vec![0],
        false => text
            .split(';')
            .map(|part| part.parse().unwrap_or(0))
            .collect(),
    };
    let mut index = 0;
    while let Some(&code) = params.get(index) {
        match code {
            0 => *state = State::default(),
            1 => state.bold = true,
            2 => state.dim = true,
            3 => state.italic = true,
            4 => state.underline = true,
            9 => state.strikethrough = true,
            22 => (state.bold, state.dim) = (false, false),
            23 => state.italic = false,
            24 => state.underline = false,
            29 => state.strikethrough = false,
            39 => state.fg = None,
            49 => state.bg = None,
            30..=37 => state.fg = Some(STANDARD[code as usize - 30].to_owned()),
            40..=47 => state.bg = Some(STANDARD[code as usize - 40].to_owned()),
            90..=97 => state.fg = Some(BRIGHT[code as usize - 90].to_owned()),
            100..=107 => state.bg = Some(BRIGHT[code as usize - 100].to_owned()),
            38 | 48 => {
                let color = match params.get(index + 1) {
                    Some(5) => {
                        index += 2;
                        params.get(index).and_then(|&n| create_color(n))
                    }
                    Some(2) => {
                        let channel = |offset| params.get(index + offset).copied();
                        let rgb = (channel(2), channel(3), channel(4));
                        index += 4;
                        let (r, g, b) =
                            (rgb.0.unwrap_or(0), rgb.1.unwrap_or(0), rgb.2.unwrap_or(0));
                        Some(format!(
                            "#{:02x}{:02x}{:02x}",
                            r.min(255),
                            g.min(255),
                            b.min(255)
                        ))
                    }
                    _ => None,
                };
                match code {
                    38 => state.fg = color,
                    _ => state.bg = color,
                }
            }
            _ => {}
        }
        index += 1;
    }
}

// The 256-color palette: the 16 above, a 6×6×6 cube, then 24 grays
fn create_color(n: u32) -> Option<String> {
    match n {
        0..=7 => Some(STANDARD[n as usize].to_owned()),
        8..=15 => Some(BRIGHT[n as usize - 8].to_owned()),
        16..=231 => {
            let level = |v: u32| if v == 0 { 0 } else { 55 + v * 40 };
            let index = n - 16;
            let (r, g, b) = (index / 36, index % 36 / 6, index % 6);
            Some(format!("#{:02x}{:02x}{:02x}", level(r), level(g), level(b)))
        }
        232..=255 => {
            let v = 8 + (n - 232) * 10;
            Some(format!("#{v:02x}{v:02x}{v:02x}"))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "ansi_unit.rs"]
mod unit;
