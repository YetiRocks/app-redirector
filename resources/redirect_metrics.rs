use yeti_sdk::prelude::*;

/// Analytics about redirect rules
#[derive(Default)]
pub struct RedirectMetrics;

impl Resource for RedirectMetrics {
    fn name(&self) -> &str { "redirectmetrics" }

    get!(request, ctx, {
        let request_id = request.id();
        let client_ip = request.ip().unwrap_or_else(|| "unknown".to_string());
        let hostname = request.hostname().unwrap_or_else(|| "unknown".to_string());

        tracing::info!(
            "Metrics request: id={}, ip={}, host={}",
            request_id, client_ip, hostname
        );

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

        reply().json(json!({
            "totalRules": total,
            "activeRules": total,
            "byHost": by_host,
            "byStatusCode": by_status,
            "_meta": {
                "requestId": request_id,
                "clientIp": client_ip
            }
        }))
    });

    post!(request, ctx, {
        let request_id = request.id();
        let client_ip = request.ip().unwrap_or_else(|| "unknown".to_string());
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
    });

    put!(request, _ctx, {
        let request_id = request.id();
        reply()
            .header("x-request-id", &request_id)
            .messagepack(json!({"format": "messagepack", "efficient": true}))
    });

    patch!(request, _ctx, {
        let request_id = request.id();
        reply()
            .header("x-request-id", &request_id)
            .cbor(json!({"format": "cbor", "standard": "RFC 8949"}))
    });
}

register_resource!(RedirectMetrics);
