use yeti_sdk::prelude::*;
use yeti_sdk::utils::redirect::{apply_query_string_mode, lookup_rule, normalize_path};

/// Check for matching redirect rule (returns JSON for edge worker integration)
///
/// Query params:
/// - url (or path ID): The path to check
/// - h: Host filter (optional)
/// - v: Version number (default: 0)
/// - qs: Query string mode - 'i' to ignore, 'm' to match (default: 'm')
#[derive(Default)]
pub struct CheckRedirect;

impl Resource for CheckRedirect {
    fn name(&self) -> &str { "checkredirect" }

    get!(request, ctx, {
        let Some(raw_path) = ctx.id().or_else(|| ctx.get("url")) else {
            return reply().json(json!(null));
        };

        let path = normalize_path(raw_path);
        let host = ctx.get_str("h", "").trim().to_lowercase();
        let version = ctx.get_i64("v", 0);
        let qs_mode = ctx.get_str("qs", "m");
        let search_path = apply_query_string_mode(&path, &qs_mode);
        let rules = ctx.tables()?.get("Rule")?;

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

        reply().json(json!(null))
    });
}

register_resource!(CheckRedirect);
