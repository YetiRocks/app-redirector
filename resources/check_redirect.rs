use yeti_core::prelude::*;
use yeti_core::utils::redirect::{apply_query_string_mode, lookup_rule, normalize_path};

/// Check for matching redirect rule (returns JSON for edge worker integration)
///
/// This endpoint returns the redirect rule as a JSON object, allowing
/// edge workers (e.g., Akamai, Cloudflare) to perform the actual HTTP redirect.
///
/// Query params:
/// - url (or path ID): The path to check for redirects
/// - h: Host filter (optional)
/// - v: Version number (default: 0)
/// - qs: Query string mode - 'i' to ignore query string, 'm' to match (default: 'm')
///
/// Returns:
/// - JSON object with redirect rule if found and active
/// - null if no matching rule or rule is outside time window
#[derive(Default)]
pub struct CheckRedirect;

impl Resource for CheckRedirect {
    fn name(&self) -> &str { "checkredirect" }
    fn is_public(&self) -> bool { true }

    get!(request, ctx, {
        // Get the path from ID or url parameter
        let Some(raw_path) = ctx.id().or_else(|| ctx.get("url")) else {
            return reply().json(json!(null));
        };

        // Normalize and process the path
        let path = normalize_path(raw_path);
        let host = ctx.get_str("h", "").trim().to_lowercase();
        let version = ctx.get_i64("v", 0);
        let qs_mode = ctx.get_str("qs", "m");

        // Apply query string mode
        let search_path = apply_query_string_mode(&path, &qs_mode);

        // Get the rules table
        let rules = ctx.tables()?.get("Rule")?;

        // Lookup with time checking and host fallback
        if let Some(record) = lookup_rule(&rules, version, &host, search_path).await? {
            return reply().json(json!({
                "path": record.get("path"),
                "host": record.get("host"),
                "redirectURL": record.get("redirectURL"),
                "statusCode": record.get("statusCode"),
                "version": record.get("version"),
                "regex": record.get("regex"),
                "utcStartTime": record.get("utcStartTime"),
                "utcEndTime": record.get("utcEndTime"),
            }));
        }

        // No matching active rule found
        reply().json(json!(null))
    });
}

register_resource!(CheckRedirect);
