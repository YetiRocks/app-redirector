use yeti_sdk::prelude::*;

/// The role that may upload redirect rules (`auth/roles.json`).
// Compared as the full `{app}:{role}` id: `has_role` compares bare names, so
// another app's role of the same bare name would pass.
const ROLE_EDITOR: &str = "app-redirector:editor";

// Upload redirect rules via CSV or JSON.
resource!(RedirectUpload {
    name = "redirectupload",
    post(ctx) => {
        // Writes `Rule`, so it needs an authenticated caller holding the
        // `editor` role (auth/roles.json grants it the write). Refused before
        // any work; the upsert runs on the caller's handle, so the role's
        // table grant is enforced again by the store (YTC-1905).
        if !ctx.access().is_authenticated() {
            return unauthorized("POST /app-redirector/api/redirectupload needs an authenticated caller");
        }
        if !(ctx.access().is_super_user() || ctx.access().role() == ROLE_EDITOR) {
            return error_response(403, "POST /app-redirector/api/redirectupload needs the app-redirector:editor role");
        }
        let is_csv = ctx.headers().get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.contains("csv"))
            .unwrap_or(false);

        let redirects: Vec<Value> = if is_csv {
            parse_csv(ctx.body())
        } else {
            match ctx.require_json_body()?.clone() {
                Value::Array(arr) => arr,
                obj => vec![obj],
            }
        };

        let rules = ctx.table("Rule")?;

        let result = bulk_upsert(
            &rules,
            redirects,
            |item| {
                let path = item["path"].as_str()?.trim().to_lowercase();
                let host = item["host"].as_str().unwrap_or("").trim().to_lowercase();
                let version = item["version"].as_i64().unwrap_or(0);
                Some(composite_key_from(&[&version as &dyn std::fmt::Display, &host, &path]))
            },
            |item| {
                let path = item["path"].as_str()
                    .map(|s| s.trim().to_lowercase())
                    .ok_or("missing path".to_string())?;
                let url = item["redirectURL"].as_str()
                    .map(|s| s.trim().to_lowercase())
                    .ok_or("missing redirectURL".to_string())?;
                let host = item["host"].as_str().unwrap_or("").trim().to_lowercase();
                let status = item["statusCode"].as_i64().unwrap_or(301);
                let version = item["version"].as_i64().unwrap_or(0);
                let regex = item["regex"].as_bool().unwrap_or(false);

                Ok(json!({
                    "path": path, "host": host, "redirectURL": url,
                    "statusCode": status, "version": version, "regex": regex
                }))
            },
        ).await?;

        ok(result.to_json("Successfully loaded"))
    }
});
