use crate::actions::page::read::{create_route_path, create_url, read_date};
use crate::models::config::PatchArticle;
use crate::models::page::Page;

/// The page with every field a patch sets replacing its own; an invalid `path` or `date`
/// is an error naming the field.
pub(crate) fn patch_page(page: Page, article: &PatchArticle) -> Result<Page, String> {
    let mut page = page;
    if let Some(path) = &article.path {
        page.path = create_route_path(path)?;
        page.url = create_url(&page.path);
    }
    if let Some(date) = &article.date {
        let parsed = read_date(date);
        page.date = Some(
            parsed.ok_or_else(|| format!("date must be YYYY-MM-DD, got \"{date}\""))?,
        );
    }
    if article.toc == Some(false) {
        page.toc.clear();
    }
    let set = |value: &Option<String>, field: &mut Option<String>| {
        if value.is_some() {
            field.clone_from(value);
        }
    };
    set(&article.label, &mut page.label);
    set(&article.description, &mut page.description);
    set(&article.icon, &mut page.icon);
    set(&article.image, &mut page.image);
    set(&article.source_url, &mut page.source_url);
    if let Some(title) = &article.title {
        page.title.clone_from(title);
    }
    if !article.author.is_empty() {
        page.authors.clone_from(&article.author);
    }
    if let Some(tags) = &article.tags {
        page.tags.clone_from(tags);
    }
    page.order = article.order.or(page.order);
    page.sidebar = article.sidebar.unwrap_or(page.sidebar);
    page.breadcrumbs = article.breadcrumbs.or(page.breadcrumbs);
    page.source_action = article.source_action.unwrap_or(page.source_action);
    Ok(page)
}

#[cfg(test)]
#[path = "patch_unit.rs"]
mod unit;
