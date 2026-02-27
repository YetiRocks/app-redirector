use yeti_core::prelude::*;

/// Upload redirect rules via CSV or JSON
#[derive(Default)]
pub struct RedirectUpload;

impl Resource for RedirectUpload {
    fn name(&self) -> &str { "redirectupload" }
    fn is_public(&self) -> bool { true }

    post!(request, ctx, {
        let is_csv = ctx.content_type()
            .map(|ct| ct.contains("csv"))
            .unwrap_or(false);

        let redirects: Vec<serde_json::Value> = if is_csv {
            parse_csv(request.body())
        } else {
            match request.json_value()? {
                serde_json::Value::Array(arr) => arr,
                obj => vec![obj],
            }
        };

        let rules = ctx.get_table("Rule")?;
        let (success, skipped) = process_redirects(&rules, redirects).await?;

        reply().json(json!({
            "message": format!("Successfully loaded {} redirects", success),
            "skipped": skipped
        }))
    });
}

fn parse_csv(body: &[u8]) -> Vec<serde_json::Value> {
    let content = String::from_utf8_lossy(body);
    let mut lines = content.lines();
    let headers: Vec<&str> = lines.next().unwrap_or("").split(',').map(|s| s.trim()).collect();

    lines.filter_map(|line| {
        if line.trim().is_empty() { return None; }
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let obj: serde_json::Map<_, _> = headers.iter().enumerate()
            .filter_map(|(i, h)| values.get(i).map(|v| (h.to_string(), json!(v.to_string()))))
            .collect();
        Some(serde_json::Value::Object(obj))
    }).collect()
}

async fn process_redirects(
    table: &Table,
    redirects: Vec<serde_json::Value>,
) -> Result<(usize, Vec<serde_json::Value>)> {
    let mut success = 0;
    let mut skipped = Vec::new();

    for item in redirects {
        let Some(path) = item["path"].as_str().map(|s| s.trim().to_lowercase()) else {
            skipped.push(json!({"reason": "missing path", "item": item}));
            continue;
        };
        let Some(url) = item["redirectURL"].as_str().map(|s| s.trim().to_lowercase()) else {
            skipped.push(json!({"reason": "missing redirectURL", "item": item}));
            continue;
        };

        let host = item["host"].as_str().unwrap_or("").trim().to_lowercase();
        let status = item["statusCode"].as_i64().unwrap_or(301);
        let version = item["version"].as_i64().unwrap_or(0);
        let regex = item["regex"].as_bool().unwrap_or(false);

        let key = format!("{}||{}||{}", version, host, path);

        // Check if already exists
        if table.get_by_id(&key).await?.is_some() {
            skipped.push(json!({"reason": "duplicate", "item": item}));
            continue;
        }

        let record = json!({
            "path": path, "host": host, "redirectURL": url,
            "statusCode": status, "version": version, "regex": regex
        });
        table.put(&key, record).await?;
        success += 1;
    }

    Ok((success, skipped))
}

register_resource!(RedirectUpload);
