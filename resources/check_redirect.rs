use yeti_sdk::prelude::*;
use yeti_sdk::utils::redirect::{apply_query_string_mode, lookup_rule, normalize_path};

// Check for matching redirect rule (returns JSON for edge worker integration).
// Query params: url (or path ID), h (host filter, optional), v (version, default 0),
// qs (query string mode — 'i' to ignore, 'm' to match, default 'm').
resource!(CheckRedirect {
    name = "checkredirect",
    get(ctx) => {
        let path_opt = (!ctx.path_id.is_empty()).then(|| ctx.path_id.as_str());
        let Some(raw_path) = path_opt.or_else(|| ctx.query("url")) else {
            return ok(json!(null));
        };

        let path = normalize_path(raw_path);
        let host = ctx.query("h").unwrap_or("").trim().to_lowercase();
        let version = ctx.query_int("v", 0);
        let qs_mode = ctx.query("qs").unwrap_or("m");
        let search_path = apply_query_string_mode(&path, &qs_mode);
        let rules = ctx.tables()?.get("Rule")?;

        if let Some(record) = lookup_rule(&rules, version, &host, search_path).await? {
            return ok(json!({
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

        ok(json!(null))
    }
});
