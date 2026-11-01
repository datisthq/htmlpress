/// A page's served path as Livemark routes it: every folder and the file's name slugified,
/// `User Guide/Getting Started.es.html` → `user-guide/getting-started.es.html`. The name's
/// dot parts are slugified one by one, so a language suffix stays a suffix.
pub(crate) fn slugify_path(path: &str) -> String {
    let (folders, name) = path.rsplit_once('/').unwrap_or(("", path));
    let (stem, extension) = name.rsplit_once('.').unwrap_or((name, ""));
    let stem: Vec<String> = stem.split('.').map(slugify_segment).collect();
    let name = match extension {
        "" => stem.join("."),
        extension => format!("{}.{extension}", stem.join(".")),
    };
    (folders.split('/').filter(|folder| !folder.is_empty()))
        .map(slugify_segment)
        .chain([name])
        .collect::<Vec<_>>()
        .join("/")
}

// A segment of symbols alone keeps its name, where Livemark's would vanish into its parent
fn slugify_segment(segment: &str) -> String {
    let mut words = vec![String::new()];
    let mut previous: Option<char> = None;
    let chars: Vec<char> = segment.chars().collect();
    for (index, char) in chars.iter().copied().enumerate() {
        if !char.is_alphanumeric() {
            words.push(String::new());
            previous = None;
            continue;
        }
        let next = chars.get(index + 1).copied();
        let after_lower = previous.is_some_and(|it| it.is_lowercase() || it.is_numeric());
        let ends_acronym = previous.is_some_and(char::is_uppercase)
            && next.is_some_and(char::is_lowercase);
        if char.is_uppercase() && (after_lower || ends_acronym) {
            words.push(String::new());
        }
        if let Some(word) = words.last_mut() {
            word.extend(char.to_lowercase());
        }
        previous = Some(char);
    }
    let slug = (words.iter().filter(|word| !word.is_empty()))
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("-");
    match slug.is_empty() {
        true => segment.to_owned(),
        false => slug,
    }
}

#[cfg(test)]
#[path = "slugify_unit.rs"]
mod unit;
