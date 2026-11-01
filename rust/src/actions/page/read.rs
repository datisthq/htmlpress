use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use lol_html::html_content::ContentType;
use lol_html::html_content::Element;
use lol_html::{RewriteStrSettings, doc_text, doctype, element, rewrite_str, text};
use textlane::{Allocator, File};

use crate::actions::icon::render::{list_icons, render_icon};
use crate::actions::package::convert::{MANAGERS, convert_command};
use crate::actions::pipeline::create::create_pipeline;
use crate::models::error::Error;
use crate::models::page::{Heading, Page, SourceAction};
use crate::models::pipeline::{Pipeline, PipelineOptions};
use crate::settings::NOT_FOUND_PAGE;

thread_local! {
    static DEFAULT: RefCell<Result<Pipeline, String>> =
        RefCell::new(create_pipeline(&PipelineOptions::default()));
}

/// Split one HTML source into a [`Page`]. `path` is relative to the project root. A full
/// document and a bare fragment both work: the wrappers are dropped, the content kept.
pub fn read_page(path: &str, html: &str) -> crate::Result<Page> {
    let mut page = read_source(path, html)?;
    DEFAULT.with_borrow_mut(|pipeline| match pipeline {
        Ok(pipeline) => finish_page(&mut page, pipeline),
        Err(message) => Err(Error::page(path)(message.clone())),
    })?;
    Ok(page)
}

/// A page's facts, read from its source without parsing it: title, metas, URL. Its
/// `content` holds the source until [`finish_page`] makes it the page.
pub(crate) fn read_source(path: &str, html: &str) -> crate::Result<Page> {
    let facts = read_facts(html).map_err(Error::page(path))?;
    let output = match &facts.route {
        Some(route) => create_route_path(route).map_err(Error::page(path))?,
        None => path.to_owned(),
    };
    let title = [facts.title, facts.heading]
        .into_iter()
        .map(|text| decode_entities(text.trim()))
        .find(|text| !text.is_empty())
        .unwrap_or_else(|| read_stem(path));
    Ok(Page {
        url: create_url(&output),
        path: output,
        source: path.to_owned(),
        key: String::new(),
        language: facts.language,
        untranslated: false,
        title,
        label: facts.label,
        icon: facts.icon,
        sidebar: facts.sidebar,
        breadcrumbs: facts.breadcrumbs,
        source_url: facts.source_url,
        source_action: facts.source_action,
        description: facts.description,
        date: facts.date,
        authors: facts.authors,
        tags: facts.tags,
        updated: None,
        image: facts.image,
        order: facts.order,
        layout: facts.layout,
        head: String::new(),
        content: html.to_owned(),
        toc: vec![],
        markdown: String::new(),
    })
}

/// Make a page read by [`read_source`] whole, parsing its source once: the pipeline's
/// prose, math, heading ids, links and code colors, then its content and extra head tags,
/// its table of contents and its Markdown copy, all from that one tree.
pub(crate) fn finish_page(page: &mut Page, pipeline: &mut Pipeline) -> crate::Result<()> {
    let path = page.source.clone();
    let fail = |message: String| Error::page(&path)(message);
    let source = std::mem::take(&mut page.content);
    let prepared = prepare_content(&source).map_err(fail)?;
    let document = is_document(&source);
    let failed = |error: textlane::Error| error.message;
    let processor = match document {
        true => pipeline.document.freeze(),
        false => pipeline.fragment.freeze(),
    };
    let processor = processor.map_err(failed).map_err(fail)?;
    let alloc = Allocator::new();
    let mut file = File::new();
    file.path = Some(page.source.clone());
    let tree = processor
        .parse(&prepared, &mut file, &alloc)
        .map_err(failed)
        .map_err(fail)?;
    let tree = processor
        .run_sync(tree, &mut file, &alloc)
        .map_err(failed)
        .map_err(fail)?;
    // KaTeX reports bad LaTeX as a message and draws it red; a page must not ship that
    let math = file.messages.iter().find(|message| {
        matches!(message.rule.as_deref(), Some("parseerror" | "rangeerror"))
    });
    if let Some(message) = math {
        return Err(fail(format!("invalid math: {}", message.reason)));
    }
    let html = processor
        .stringify(&tree, &mut file)
        .map_err(failed)
        .map_err(fail)?;
    let html = name_references(&html);
    let facts = read_facts(&html).map_err(fail)?;
    let headings = create_ids(facts.headings);
    page.content = read_content(&html, &headings, document).map_err(fail)?;
    page.head = match document {
        true => read_head(&html).map_err(fail)?,
        false => read_fragment_head(&html).map_err(fail)?,
    };
    page.toc = match facts.toc {
        true => (headings.into_iter())
            .filter(|(_, listed)| *listed)
            .map(|(heading, _)| heading)
            .collect(),
        false => vec![],
    };
    let markdown = pipeline.markdown.freeze().map_err(failed).map_err(fail)?;
    let mut file = File::new();
    let tree = (markdown.run_sync(tree, &mut file, &alloc))
        .map_err(failed)
        .map_err(fail)?;
    let body = (markdown.stringify(&tree, &mut file))
        .map_err(failed)
        .map_err(fail)?;
    let mut parts = vec![format!("# {}", page.title)];
    if let Some(description) = &page.description {
        parts.push(format!("> {description}"));
    }
    if !body.trim().is_empty() {
        parts.push(body.trim().to_owned());
    }
    page.markdown = format!("{}\n", parts.join("\n\n"));
    Ok(())
}

#[derive(Default)]
struct Facts {
    title: String,
    heading: String,
    description: Option<String>,
    label: Option<String>,
    icon: Option<String>,
    route: Option<String>,
    sidebar: bool,
    breadcrumbs: Option<bool>,
    toc: bool,
    source_url: Option<String>,
    source_action: SourceAction,
    date: Option<String>,
    authors: Vec<String>,
    tags: Vec<String>,
    image: Option<String>,
    order: Option<i64>,
    layout: Option<String>,
    language: String,
    has_head: bool,
    /// Each heading, and whether it belongs in the table of contents.
    headings: Vec<(Heading, bool)>,
}

fn read_facts(html: &str) -> Result<Facts, String> {
    let facts = RefCell::new(Facts {
        sidebar: true,
        toc: true,
        ..Facts::default()
    });
    let headings = Cell::new(0);
    let mut settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("head", |_| {
            facts.borrow_mut().has_head = true;
            Ok(())
        }))
        .append_element_content_handler(text!("title", |chunk| {
            facts.borrow_mut().title.push_str(chunk.as_str());
            Ok(())
        }))
        .append_element_content_handler(element!("h1", |_| {
            headings.set(headings.get() + 1);
            Ok(())
        }))
        .append_element_content_handler(text!("h1", |chunk| {
            if headings.get() == 1 {
                facts.borrow_mut().heading.push_str(chunk.as_str());
            }
            Ok(())
        }))
        .append_element_content_handler(element!("h2, h3, h4, h5, h6", |el| {
            let heading = Heading {
                id: el.get_attribute("id").unwrap_or_default(),
                title: String::new(),
                level: (el.tag_name().as_bytes().get(1)).map_or(2, |digit| digit - b'0'),
            };
            // `data-toc="false"` keeps a heading off the table of contents; "only" keeps it
            // there but off the page, which the theme collapses to nothing in place
            let listed = match el.get_attribute("data-toc").as_deref().map(str::trim) {
                None | Some("only" | "true") => true,
                Some("false") => false,
                Some(other) => {
                    return Err(format!(
                        "data-toc on a heading must be false or only, got \"{other}\""
                    )
                    .into());
                }
            };
            // An <h5> or <h6> gets an id and an anchor, never a table of contents entry
            let listed = listed && heading.level <= 4;
            facts.borrow_mut().headings.push((heading, listed));
            Ok(())
        }))
        .append_element_content_handler(text!("h2, h3, h4, h5, h6", |chunk| {
            if let Some((heading, _)) = facts.borrow_mut().headings.last_mut() {
                heading.title.push_str(chunk.as_str());
            }
            Ok(())
        }));
    // Runs after the handler above on the same heading: a card's or a step's title is
    // part of that block, not a section of the page
    for selector in UNLISTED_HEADINGS {
        settings = settings.append_element_content_handler(element!(selector, |_| {
            if let Some((_, listed)) = facts.borrow_mut().headings.last_mut() {
                *listed = false;
            }
            Ok(())
        }));
    }
    settings = settings
        .append_element_content_handler(element!("meta[name='description']", |el| {
            facts.borrow_mut().description = el.get_attribute("content");
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='date']", |el| {
            let date = el.get_attribute("content").unwrap_or_default();
            facts.borrow_mut().date = Some(read_date(&date).ok_or_else(|| {
                format!("<meta name=\"date\"> must be a YYYY-MM-DD date, got \"{date}\"")
            })?);
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='author']", |el| {
            let author = el.get_attribute("content").unwrap_or_default();
            if !author.trim().is_empty() {
                facts.borrow_mut().authors.push(author.trim().to_owned());
            }
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='keywords']", |el| {
            let keywords = el.get_attribute("content").unwrap_or_default();
            facts.borrow_mut().tags = (keywords.split(','))
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_owned)
                .collect();
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='image']", |el| {
            facts.borrow_mut().image = el.get_attribute("content");
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='order']", |el| {
            let order = el.get_attribute("content").unwrap_or_default();
            let order = order.trim().parse().map_err(|_| {
                format!("<meta name=\"order\"> must be an integer, got \"{order}\"")
            })?;
            facts.borrow_mut().order = Some(order);
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='label']", |el| {
            facts.borrow_mut().label = read_content_attribute(el);
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='icon']", |el| {
            let icon = el.get_attribute("content").unwrap_or_default();
            if render_icon(icon.trim()).is_none() {
                return Err(format!("<meta name=\"icon\">: no built-in icon \"{icon}\"").into());
            }
            facts.borrow_mut().icon = Some(icon.trim().to_owned());
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='path']", |el| {
            facts.borrow_mut().route = read_content_attribute(el);
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='language']", |el| {
            facts.borrow_mut().language = read_content_attribute(el).unwrap_or_default();
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='sidebar']", |el| {
            facts.borrow_mut().sidebar = read_flag(el, "sidebar")?;
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='breadcrumbs']", |el| {
            facts.borrow_mut().breadcrumbs = Some(read_flag(el, "breadcrumbs")?);
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='toc']", |el| {
            facts.borrow_mut().toc = read_flag(el, "toc")?;
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='source-url']", |el| {
            facts.borrow_mut().source_url = read_content_attribute(el);
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='source-action']", |el| {
            let action = el.get_attribute("content").unwrap_or_default();
            facts.borrow_mut().source_action = match action.trim() {
                "edit" => SourceAction::Edit,
                "view" => SourceAction::View,
                _ => {
                    return Err(format!(
                        "<meta name=\"source-action\"> must be edit or view, got \"{action}\""
                    )
                    .into());
                }
            };
            Ok(())
        }))
        .append_element_content_handler(element!("meta[name='layout']", |el| {
            let layout = el.get_attribute("content").unwrap_or_default();
            let layout = layout.trim();
            if !is_layout_name(layout) {
                return Err(format!(
                    "<meta name=\"layout\"> must be lowercase letters, digits and \
                     hyphens, got \"{layout}\""
                )
                .into());
            }
            facts.borrow_mut().layout = Some(layout.to_owned());
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(facts.into_inner())
}

// Ids already in the page are reserved first, so a generated slug never steals one that
// a link elsewhere points at.
fn create_ids(headings: Vec<(Heading, bool)>) -> Vec<(Heading, bool)> {
    let mut taken: Vec<String> = headings
        .iter()
        .filter(|(heading, _)| !heading.id.is_empty())
        .map(|(heading, _)| heading.id.clone())
        .collect();
    headings
        .into_iter()
        .map(|(heading, listed)| {
            let title = decode_entities(
                &heading
                    .title
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            let id = match heading.id.is_empty() {
                true => create_slug(&title, &mut taken),
                false => heading.id,
            };
            (
                Heading {
                    id,
                    title,
                    ..heading
                },
                listed,
            )
        })
        .collect()
}

fn create_slug(title: &str, taken: &mut Vec<String>) -> String {
    let words: Vec<String> = title
        .to_lowercase()
        .split(|char: char| !char.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect();
    let base = match words.is_empty() {
        true => "section".to_owned(),
        false => words.join("-"),
    };
    let mut slug = base.clone();
    let mut count = 1;
    while taken.contains(&slug) {
        count += 1;
        slug = format!("{base}-{count}");
    }
    taken.push(slug.clone());
    slug
}

const UNLISTED_HEADINGS: &[&str] = &[
    "htmlpress-callout h2",
    "htmlpress-callout h3",
    "htmlpress-callout h4",
    "htmlpress-card h2",
    "htmlpress-card h3",
    "htmlpress-card h4",
    "h2[data-step]",
    "h3[data-step]",
    "h4[data-step]",
    "htmlpress-tabs h2",
    "htmlpress-tabs h3",
    "htmlpress-tabs h4",
];

// What becomes plain HTML before the parse: package tabs into tabs of code blocks, so they
// are colored like any other, icons into their SVG, and themed images into the attribute
fn prepare_content(html: &str) -> Result<String, String> {
    let commands = Rc::new(RefCell::new(String::new()));
    let titles = Rc::new(RefCell::new(VecDeque::<String>::new()));
    let tab_titles = titles.clone();
    let settings = RewriteStrSettings::new()
        // Expanded here, not in the browser, so the tabs component and the copy buttons
        // treat each manager's command like any other code block
        .append_element_content_handler(element!("htmlpress-package-tabs", |el| {
            el.set_tag_name("htmlpress-tabs")?;
            if !el.has_attribute("sync") {
                el.set_attribute("sync", "package-manager")?;
            }
            commands.borrow_mut().clear();
            let commands = commands.clone();
            let handler: lol_html::EndTagHandler<'static> = Box::new(move |end| {
                let commands = commands.borrow();
                end.before(&create_package_tabs(commands.trim()), ContentType::Html);
                Ok(())
            });
            el.on_end_tag(handler)
        }))
        .append_element_content_handler(element!("htmlpress-icon", |el| {
            let name = el.get_attribute("name").unwrap_or_default();
            let svg = render_icon(&name).ok_or_else(|| {
                format!(
                    "<htmlpress-icon name=\"{name}\">: no such icon; the icons are: {}",
                    list_icons()
                )
            })?;
            el.set_inner_content(&svg, ContentType::Html);
            Ok(())
        }))
        // A Markdown run of `tab="…"` fences arrives as one block per tab and their titles
        .append_element_content_handler(element!("htmlpress-tabs[tabs]", |el| {
            let list = el.get_attribute("tabs").unwrap_or_default();
            let list = list.replace("&#x22;", "\"").replace("&quot;", "\"");
            let list: Vec<String> = serde_json::from_str(&list)
                .map_err(|error| format!("<htmlpress-tabs tabs>: {error}"))?;
            *titles.borrow_mut() = list.into();
            el.remove_attribute("tabs");
            Ok(())
        }))
        .append_element_content_handler(element!("htmlpress-tabs > pre", |el| {
            if let Some(title) = tab_titles.borrow_mut().pop_front() {
                let title = title.replace('&', "&amp;").replace('"', "&quot;");
                el.before(
                    &format!("<htmlpress-tab title=\"{title}\">"),
                    ContentType::Html,
                );
                el.after("</htmlpress-tab>", ContentType::Html);
            }
            Ok(())
        }))
        .append_element_content_handler(element!("details[summary]", |el| {
            let summary = el.get_attribute("summary").unwrap_or_default();
            el.prepend(&format!("<summary>{summary}</summary>"), ContentType::Html);
            el.remove_attribute("summary");
            Ok(())
        }))
        .append_element_content_handler(element!("htmlpress-card[title]", |el| {
            let heading = create_card_heading(el)?;
            el.prepend(&heading, ContentType::Html);
            for name in ["title", "href", "icon"] {
                el.remove_attribute(name);
            }
            Ok(())
        }))
        .append_element_content_handler(element!("htmlpress-button", |el| {
            let (href, label) = (el.get_attribute("href"), el.get_attribute("label"));
            let (Some(href), Some(label)) = (href, label) else {
                return Err("<htmlpress-button> needs an href and a label".into());
            };
            let link = format!("<a href=\"{}\">{label}</a>", href.replace('"', "&quot;"));
            el.set_inner_content(&link, ContentType::Html);
            el.remove_attribute("href");
            el.remove_attribute("label");
            Ok(())
        }))
        .append_element_content_handler(element!("htmlpress-video", |el| {
            write_media_link(el, "htmlpress-video", "youtube")
        }))
        .append_element_content_handler(element!("htmlpress-audio", |el| {
            write_media_link(el, "htmlpress-audio", "soundcloud")
        }))
        .append_element_content_handler(element!("htmlpress-package-tabs *", |el| {
            el.remove_and_keep_content();
            Ok(())
        }))
        .append_element_content_handler(text!("htmlpress-package-tabs", |chunk| {
            commands.borrow_mut().push_str(chunk.as_str());
            chunk.remove();
            Ok(())
        }))
        // Livemark's `#light` and `#dark` image suffixes, as the attribute the theme reads
        .append_element_content_handler(element!("img[src]", |el| {
            let src = el.get_attribute("src").unwrap_or_default();
            for theme in ["light", "dark"] {
                if let Some(bare) = src.strip_suffix(&format!("#{theme}")) {
                    el.set_attribute("src", bare)?;
                    el.set_attribute("data-theme", theme)?;
                }
            }
            Ok(())
        }));
    let html = wrap_mermaid(html);
    rewrite_str(&html, settings).map_err(|error| error.to_string())
}

// Livemark's diagrams are fences, so a `language-mermaid` block becomes the diagram;
// on the string, since a rewriter's handler for `<pre>` cannot see the code inside it
fn wrap_mermaid(html: &str) -> String {
    const OPEN: &str = "<pre><code class=\"language-mermaid\">";
    const CLOSE: &str = "</code></pre>";
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find(OPEN) {
        let body = &rest[start + OPEN.len()..];
        let Some(end) = body.find(CLOSE) else { break };
        out.push_str(&rest[..start]);
        out.push_str("<htmlpress-mermaid><pre>");
        out.push_str(&body[..end]);
        out.push_str("</pre></htmlpress-mermaid>");
        rest = &body[end + CLOSE.len()..];
    }
    out.push_str(rest);
    out
}

// Written here, not in the browser, so the heading and its link reach search and the
// Markdown copy; the icon as its SVG, since this pass never sees what it inserts
fn create_card_heading(el: &Element) -> Result<String, String> {
    let title = el.get_attribute("title").unwrap_or_default();
    let icon = match el.get_attribute("icon") {
        Some(name) => {
            let svg = render_icon(&name).ok_or_else(|| {
                format!(
                    "<htmlpress-card icon=\"{name}\">: no such icon; the icons are: {}",
                    list_icons()
                )
            })?;
            format!("<htmlpress-icon name=\"{name}\">{svg}</htmlpress-icon>")
        }
        None => String::new(),
    };
    Ok(match el.get_attribute("href") {
        Some(href) => format!(
            "<h3><a href=\"{}\">{icon}{title}</a></h3>",
            href.replace('"', "&quot;")
        ),
        None => format!("<h3>{icon}{title}</h3>"),
    })
}

// The embed is the script's; the link it starts as keeps a page without it pointing there
fn write_media_link(
    el: &mut Element,
    tag: &str,
    kind: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let found = el.get_attribute("type").unwrap_or_default();
    if found != kind {
        return Err(format!("<{tag}> needs type=\"{kind}\", got \"{found}\"").into());
    }
    let url = el.get_attribute("url").unwrap_or_default();
    if url.is_empty() {
        return Err(format!("<{tag} type=\"{kind}\"> needs a url").into());
    }
    let url = url.replace('"', "&quot;");
    el.set_inner_content(&format!("<a href=\"{url}\">{url}</a>"), ContentType::Html);
    Ok(())
}

// textlane writes `&`, `<`, `"` and `'` as hexadecimal references; htmlpress reads
// attribute values as written, so a `&#x27;` would show in a description, escaped twice
fn name_references(html: &str) -> String {
    html.replace("&#x26;", "&amp;")
        .replace("&#x3C;", "&lt;")
        .replace("&#x22;", "&quot;")
        .replace("&#x27;", "'")
}

// A page with its own `<html>` or `<head>` is a document; anything else a fragment, whose
// leading `<title>` and metas stay where they are
fn is_document(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    ["<html", "<head"].iter().any(|tag| {
        lower.match_indices(tag).any(|(at, _)| {
            (lower.as_bytes().get(at + tag.len())).is_some_and(|next| {
                matches!(next, b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/')
            })
        })
    })
}

fn read_content(
    html: &str,
    headings: &[(Heading, bool)],
    has_head: bool,
) -> Result<String, String> {
    let index = Cell::new(0);
    let lead = Rc::new(Lead::default());
    let mut settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("h2, h3, h4, h5, h6", |el| {
            if let Some((heading, _)) = headings.get(index.get())
                && el.get_attribute("id").is_none_or(|id| id.is_empty())
            {
                el.set_attribute("id", &heading.id)?;
            }
            index.set(index.get() + 1);
            Ok(())
        }))
        .append_document_content_handler(doctype!(|doctype| {
            doctype.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("head", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("html", |el| {
            el.remove_and_keep_content();
            Ok(())
        }))
        .append_element_content_handler(element!("body", |el| {
            el.remove_and_keep_content();
            Ok(())
        }));
    if !has_head {
        let text_lead = lead.clone();
        settings = settings
            .append_element_content_handler(element!("*", |el| {
                if read_lead(el, &lead)? {
                    el.remove();
                }
                Ok(())
            }))
            .append_document_content_handler(doc_text!(move |chunk| {
                read_lead_text(chunk.as_str(), &text_lead);
                Ok(())
            }));
    }
    // A fragment carries its title and metas without a <head>; they are read, not shown
    for selector in OWNED_BY_LAYOUT {
        settings = settings.append_element_content_handler(element!(selector, |el| {
            el.remove();
            Ok(())
        }));
    }
    let content = rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(content.trim().to_owned())
}

// The layout writes the charset, viewport, title and description itself; a second copy
// from the page would win or clash depending on the browser.
const OWNED_BY_LAYOUT: &[&str] = &[
    "title",
    "meta[charset]",
    "meta[name='viewport']",
    "meta[name='description']",
    "meta[name='image']",
    "meta[name='order']",
    "meta[name='layout']",
    "meta[name='label']",
    "meta[name='icon']",
    "meta[name='path']",
    "meta[name='language']",
    "meta[name='sidebar']",
    "meta[name='breadcrumbs']",
    "meta[name='toc']",
    "meta[name='source-url']",
    "meta[name='source-action']",
];

fn read_head(html: &str) -> Result<String, String> {
    let mut settings = RewriteStrSettings::new()
        .append_document_content_handler(doctype!(|doctype| {
            doctype.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("body", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("html > :not(head)", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("html", |el| {
            el.remove_and_keep_content();
            Ok(())
        }))
        .append_element_content_handler(element!("head", |el| {
            el.remove_and_keep_content();
            Ok(())
        }));
    for selector in OWNED_BY_LAYOUT {
        settings = settings.append_element_content_handler(element!(selector, |el| {
            el.remove();
            Ok(())
        }));
    }
    let head = rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(join_lines(&head))
}

// A browser parses the head tags a fragment starts with into <head>, so they go there
fn read_fragment_head(html: &str) -> Result<String, String> {
    let lead = Rc::new(Lead::default());
    let text_lead = lead.clone();
    let mut settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("*", |el| {
            if !read_lead(el, &lead)? {
                el.remove();
            }
            Ok(())
        }))
        .append_document_content_handler(doc_text!(move |chunk| {
            if !read_lead_text(chunk.as_str(), &text_lead) {
                chunk.remove();
            }
            Ok(())
        }));
    for selector in OWNED_BY_LAYOUT {
        settings = settings.append_element_content_handler(element!(selector, |el| {
            el.remove();
            Ok(())
        }));
    }
    let head = rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(join_lines(&head))
}

const HEAD_TAGS: &[&str] = &["title", "meta", "link", "style", "script", "base"];

#[derive(Default)]
struct Lead {
    started: Cell<bool>,
    inside: Rc<Cell<bool>>,
}

// Whether an element still belongs to the fragment's leading head tags
fn read_lead(
    el: &mut Element<'_, '_>,
    lead: &Lead,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if lead.started.get() {
        return Ok(false);
    }
    if !HEAD_TAGS.contains(&el.tag_name().as_str()) {
        lead.started.set(true);
        return Ok(false);
    }
    if el.can_have_content() {
        lead.inside.set(true);
        let inside = lead.inside.clone();
        el.on_end_tag(Box::new(move |_| {
            inside.set(false);
            Ok(())
        }))?;
    }
    Ok(true)
}

// Whether a text chunk still belongs to the leading head tags: their own text or blanks
fn read_lead_text(text: &str, lead: &Lead) -> bool {
    if !lead.started.get() && !lead.inside.get() && !text.trim().is_empty() {
        lead.started.set(true);
    }
    !lead.started.get()
}

fn join_lines(html: &str) -> String {
    html.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn create_package_tabs(commands: &str) -> String {
    let mut html = String::new();
    for manager in MANAGERS {
        let command = convert_command(commands, manager);
        html.push_str(&format!(
            "<htmlpress-tab title=\"{manager}\"><pre><code>{command}</code></pre></htmlpress-tab>"
        ));
    }
    html
}

/// A layout's name, as `<meta name="layout">` gives it and `.htmlpress/layouts/` files it.
pub(crate) fn is_layout_name(name: &str) -> bool {
    name.starts_with(|char: char| char.is_ascii_lowercase())
        && name
            .chars()
            .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-')
}

/// The `YYYY-MM-DD` a date starts with, so a full timestamp is accepted too; None when
/// it does not start with one.
pub(crate) fn read_date(text: &str) -> Option<String> {
    let date = text.trim().get(..10)?;
    let parts: Vec<&str> = date.split('-').collect();
    let valid = matches!(parts.as_slice(), [year, month, day]
        if year.len() == 4 && month.len() == 2 && day.len() == 2
            && parts.iter().all(|part| part.chars().all(|char| char.is_ascii_digit())));
    valid.then(|| date.to_owned())
}

fn read_content_attribute(el: &lol_html::html_content::Element) -> Option<String> {
    let content = el.get_attribute("content")?;
    let content = content.trim();
    (!content.is_empty()).then(|| content.to_owned())
}

fn read_flag(el: &lol_html::html_content::Element, name: &str) -> Result<bool, String> {
    match el.get_attribute("content").unwrap_or_default().trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!(
            "<meta name=\"{name}\"> must be true or false, got \"{other}\""
        )),
    }
}

/// The page path a `path` override stands for: `/intro/` → `intro/index.html`,
/// `/intro.html` → `intro.html`, both served at `/intro/`. Anything else is an error,
/// since a URL that is neither a folder nor an `.html` file could not name the page.
pub(crate) fn create_route_path(route: &str) -> Result<String, String> {
    let Some(relative) = route.strip_prefix('/') else {
        return Err(format!("path must start with \"/\", got \"{route}\""));
    };
    if relative.split('/').any(|part| part == "..") {
        return Err(format!("path must stay inside the site, got \"{route}\""));
    }
    match (
        relative.is_empty() || relative.ends_with('/'),
        relative.ends_with(".html"),
    ) {
        (true, _) => Ok(format!("{relative}index.html")),
        (false, true) => Ok(relative.to_owned()),
        (false, false) => Err(format!(
            "path must end with \"/\" or \".html\", got \"{route}\""
        )),
    }
}

/// The folder URL a page is served at: `guides/build.html` → `/guides/build/`, and
/// `guides/index.html` → `/guides/`. The 404 page keeps its file, where hosts look for it.
pub(crate) fn create_url(path: &str) -> String {
    let output = create_output_path(path);
    match output.strip_suffix("index.html") {
        Some(folder) if folder.is_empty() || folder.ends_with('/') => {
            format!("/{folder}")
        }
        _ => format!("/{output}"),
    }
}

/// The file a page is written to: `guides/build.html` → `guides/build/index.html`.
pub(crate) fn create_output_path(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    match path.strip_suffix(".html") {
        Some(stem) if name != "index.html" && path != NOT_FOUND_PAGE => {
            format!("{stem}/index.html")
        }
        _ => path.to_owned(),
    }
}

fn read_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".html").unwrap_or(name).to_owned()
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", "\u{a0}")
        .replace("&amp;", "&")
}

#[cfg(test)]
#[path = "read_unit.rs"]
mod unit;
