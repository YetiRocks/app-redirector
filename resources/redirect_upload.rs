use yeti_sdk::prelude::*;

// Upload redirect rules via CSV or JSON.
resource!(RedirectUpload {
    name = "redirectupload",
    post(ctx) => {
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
