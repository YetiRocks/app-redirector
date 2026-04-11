use yeti_sdk::prelude::*;

/// Analytics about redirect rules
resource!(RedirectMetrics {
    name = "redirectmetrics",
    get(ctx) => {
        let request_id = ctx.headers.get("x-request-id").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
        let client_ip = ctx.headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
        let hostname = ctx.headers.get("host").and_then(|v| v.to_str().ok()).unwrap_or("unknown");

        yeti_log!(info, "Metrics request: id={}, ip={}, host={}",
            request_id, client_ip, hostname);

        let rules = ctx.get_table("Rule")?;
        let mut by_host: HashMap<String, usize> = HashMap::new();
        let mut by_status: HashMap<i64, usize> = HashMap::new();

        let records: Vec<Value> = rules.get_all().await?;
        let total = records.len();

        for record in &records {
            if let Some(host) = record["host"].as_str() {
                *by_host.entry(host.to_string()).or_insert(0) += 1;
            }
            if let Some(status) = record["statusCode"].as_i64() {
                *by_status.entry(status).or_insert(0) += 1;
            }
        }

        ok(json!({
            "totalRules": total,
            "activeRules": total,
            "byHost": by_host,
            "byStatusCode": by_status,
            "_meta": {
                "requestId": request_id,
                "clientIp": client_ip
            }
        }))
    },
    post(ctx) => {
        let request_id = ctx.headers.get("x-request-id").and_then(|v| v.to_str().ok()).unwrap_or("unknown").to_string();
        let client_ip = ctx.headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()).unwrap_or("unknown").to_string();
        let rules = ctx.get_table("Rule")?;
        let records: Vec<Value> = rules.get_all().await?;

        reply()
            .header("x-request-id", &request_id)
            .header("x-client-ip", &client_ip)
            .json(json!({
                "totalRules": records.len(),
                "requestId": request_id,
                "clientIp": client_ip,
            }))
    },
    put(ctx) => {
        let request_id = ctx.headers.get("x-request-id").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
        reply()
            .header("x-request-id", request_id)
            .messagepack(json!({"format": "messagepack", "efficient": true}))
    },
    patch(ctx) => {
        let request_id = ctx.headers.get("x-request-id").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
        reply()
            .header("x-request-id", request_id)
            .cbor(json!({"format": "cbor", "standard": "RFC 8949"}))
    }
});
