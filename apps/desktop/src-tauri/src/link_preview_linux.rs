use dom_query::Document;
use reqwest::Url;

use crate::error::{AppError, AppResult};
use crate::web_fetch::{fetch_public_html, fetch_public_image};

const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;

fn image_url(html: &str, final_url: &str) -> Option<String> {
    let document = Document::from(html);
    let base = Url::parse(final_url).ok()?;
    for selector in [
        "meta[property='og:image' i]",
        "meta[name='twitter:image' i]",
    ] {
        for element in document.select(selector).iter() {
            let Some(content) = element.attr("content") else {
                continue;
            };
            let Ok(url) = base.join(content.trim()) else {
                continue;
            };
            if matches!(url.scheme(), "http" | "https")
                && url.username().is_empty()
                && url.password().is_none()
                && !content.trim().is_empty()
            {
                return Some(url.into());
            }
        }
    }
    None
}

fn validate_image_type(url: &str, content_type: &str) -> AppResult<()> {
    match content_type.split(';').next().unwrap_or("").trim() {
        "image/jpeg" | "image/png" | "image/webp" | "image/gif" => Ok(()),
        _ => Err(AppError::parse(format!(
            "{url} is not a supported preview image"
        ))),
    }
}

pub(super) async fn fetch<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    url: &str,
) -> AppResult<Option<Vec<u8>>> {
    let page = fetch_public_html(url).await?;
    let Some(image) = image_url(&String::from_utf8_lossy(&page.body), &page.final_url) else {
        return Ok(None);
    };
    let response = fetch_public_image(&image, MAX_IMAGE_BYTES, validate_image_type).await?;
    Ok(Some(response.body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_images_against_the_final_page_url() {
        let html = r#"<meta content="../cover.jpg?one=1&amp;two=2" property="OG:IMAGE">"#;
        assert_eq!(
            image_url(html, "https://example.com/posts/entry"),
            Some("https://example.com/cover.jpg?one=1&two=2".into())
        );
    }

    #[test]
    fn prefers_opengraph_and_falls_back_to_twitter() {
        let html = r#"<meta name="twitter:image" content="/twitter.png"><meta property="og:image" content="/og.png">"#;
        assert_eq!(
            image_url(html, "https://example.com"),
            Some("https://example.com/og.png".into())
        );
        let html = r#"<meta property="og:image" content="data:image/png;base64,abc"><meta name="twitter:image" content="/twitter.png">"#;
        assert_eq!(
            image_url(html, "https://example.com"),
            Some("https://example.com/twitter.png".into())
        );
    }

    #[test]
    fn ignores_missing_blank_and_credentialed_images() {
        for html in [
            "<title>Page</title>",
            r#"<meta property="og:image" content=" ">"#,
            r#"<meta property="og:image" content="https://user:password@example.com/image">"#,
        ] {
            assert_eq!(image_url(html, "https://example.com"), None);
        }
        assert!(validate_image_type("https://example.com/image", "text/html").is_err());
        assert!(validate_image_type("https://example.com/image", "image/svg+xml").is_err());
    }
}
