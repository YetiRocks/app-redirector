use yeti_sdk::prelude::*;
use yeti_sdk::utils::redirect::{lookup_rule, normalize_path};

/// Perform an actual HTTP redirect based on stored rules
///
/// Query params:
/// - h: Host filter (optional)
/// - v: Version number (default: 0)
#[derive(Default)]
pub struct PerformRedirect;

impl Resource for PerformRedirect {
    fn name(&self) -> &str { "r" }

    get!(request, ctx, {
        let uri_path = request.uri().path();
        let path = if let Some(idx) = uri_path.find("/r/") {
            let after_r = &uri_path[idx + 3..];
            if after_r.is_empty() {
                return not_found("No path specified");
            }
            normalize_path(after_r)
        } else {
            return not_found("Invalid redirect path");
        };

        let host = ctx.get_str("h", "").trim().to_lowercase();
        let version = ctx.get_i64("v", 0);
        let rules = ctx.tables()?.get("Rule")?;

        if let Some(record) = lookup_rule(&rules, version, &host, &path).await? {
            let target_url = record["redirectURL"].as_str().unwrap_or("/");
            let status_code = record["statusCode"].as_i64().unwrap_or(302) as u16;
            return build_redirect_response(target_url, status_code);
        }

        not_found(&format!("No redirect rule found for path: {}", path))
    });
}

register_resource!(PerformRedirect);

fn build_redirect_response(
    target_url: &str,
    status_code: u16,
) -> Result<Response<ResponseBody>, YetiError> {
    Response::builder()
        .status(StatusCode::from_u16(status_code).unwrap_or(StatusCode::FOUND))
        .header("Location", target_url)
        .header(
            "Cache-Control",
            if status_code == 301 || status_code == 308 {
                "public, max-age=31536000"
            } else {
                "no-cache"
            },
        )
        .body(ResponseBody::complete(vec![]))
        .map_err(|e| YetiError::Internal(format!("Failed to build redirect: {}", e)))
}
