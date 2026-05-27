use yeti_sdk::prelude::*;
use yeti_sdk::utils::redirect::{lookup_rule, normalize_path};

// Perform an actual HTTP redirect based on stored rules.
// Query params: h (host filter, optional), v (version, default 0).
resource!(PerformRedirect {
    name = "r",
    get(ctx) => {
        let uri_path = ctx.path();
        let path = if let Some(idx) = uri_path.find("/r/") {
            let after_r = &uri_path[idx + 3..];
            if after_r.is_empty() {
                return not_found("No path specified");
            }
            normalize_path(after_r)
        } else {
            return not_found("Invalid redirect path");
        };

        let host = ctx.query("h").unwrap_or("").trim().to_lowercase();
        let version = ctx.query_int("v", 0);
        let rules = ctx.tables()?.get("Rule")?;

        if let Some(record) = lookup_rule(&rules, version, &host, &path).await? {
            let target_url = record["redirectURL"].as_str().unwrap_or("/");
            let status_code = record["statusCode"].as_i64().unwrap_or(302) as u16;
            let cache = if status_code == 301 || status_code == 308 {
                "public, max-age=31536000"
            } else {
                "no-cache"
            };
            return reply()
                .header("Cache-Control", cache)
                .redirect(target_url, Some(status_code));
        }

        not_found(&format!("No redirect rule found for path: {}", path))
    }
});
