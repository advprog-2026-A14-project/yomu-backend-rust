use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ArticleValidationData {
    pub exists: bool,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JavaApiResponse {
    pub success: bool,
    pub data: Option<ArticleValidationData>,
}

/// Calls the Java Core internal endpoint to validate whether an article exists.
///
/// Returns:
/// - `Ok(Some(data))` — article exists, category info included.
/// - `Ok(None)`       — article not found in Java Core DB.
/// - `Err(e)`         — Java Core is unreachable or returned an unexpected error.
///                      Callers must treat this as a soft failure (§7.3 fault tolerance).
pub async fn validate_article(
    base_url: &str,
    api_key: &str,
    article_id: Uuid,
) -> Result<Option<ArticleValidationData>, String> {
    let url = format!("{}/api/internal/articles/{}/exists", base_url, article_id);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| format!("Gagal membuat HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()
        .await
        .map_err(|e| format!("Java Core tidak dapat dijangkau: {}", e))?;

    let status = response.status();

    if status.as_u16() == 404 {
        return Ok(None);
    }

    if !status.is_success() {
        return Err(format!(
            "Java Core membalas status tidak terduga: {}",
            status
        ));
    }

    let body: JavaApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Gagal mem-parse respons Java Core: {}", e))?;

    if !body.success {
        return Ok(None);
    }

    Ok(body.data)
}
