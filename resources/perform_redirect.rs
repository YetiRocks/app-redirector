use yeti_core::prelude::*;
use yeti_core::utils::redirect::{lookup_rule, normalize_path};

/// Perform an actual HTTP redirect based on stored rules
///
/// This resource handles requests and returns proper HTTP redirects:
/// - 301 Moved Permanently (cached for 1 year)
/// - 302 Found (Temporary Redirect, no cache)
/// - 307 Temporary Redirect (preserves method, no cache)
/// - 308 Permanent Redirect (preserves method, cached for 1 year)
/// - 404 Not Found if no matching rule or rule is outside time window
///
/// Query params:
/// - h: Host filter (optional)
/// - v: Version number (default: 0)
///
/// The path is taken from the URL path after /r/
/// Time-based rules with utcStartTime/utcEndTime are only active within that window
#[derive(Default)]
pub struct PerformRedirect;

impl Resource for PerformRedirect {
    fn name(&self) -> &str { "r" }

    fn allow_read(&self, _: &dyn AccessControl, _: &RequestTarget, _: &ResourceParams) -> bool { true }

    get!(request, ctx, {
        // Extract the full path after /r/ from the request URI
        let uri_path = request.uri().path();

        // Find /r/ in the path and extract everything after it
        let path = if let Some(idx) = uri_path.find("/r/") {
            let after_r = &uri_path[idx + 3..]; // Skip "/r/"
            if after_r.is_empty() {
                return not_found("No path specified");
            }
            normalize_path(after_r)
        } else {
            return not_found("Invalid redirect path");
        };

        let host = ctx.get_str("h", "").trim().to_lowercase();
        let version = ctx.get_i64("v", 0);

        // Get the rules table
        let rules = ctx.tables()?.get("Rule")?;

        // Lookup with time checking and host fallback
        if let Some(record) = lookup_rule(&rules, version, &host, &path).await? {
            let target_url = record["redirectURL"].as_str().unwrap_or("/");
            let status_code = record["statusCode"].as_i64().unwrap_or(302) as u16;
            return build_redirect_response(target_url, status_code);
        }

        // No matching rule or rule is outside time window - return 404
        not_found(&format!("No redirect rule found for path: {}", path))
    });
}

register_resource!(PerformRedirect);

/// Build an HTTP redirect response
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
