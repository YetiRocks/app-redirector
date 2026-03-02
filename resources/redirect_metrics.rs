use yeti_core::prelude::*;

/// Analytics about redirect rules
#[derive(Default)]
pub struct RedirectMetrics;

impl Resource for RedirectMetrics {
    fn name(&self) -> &str { "redirectmetrics" }
    fn is_public(&self) -> bool { true }

    get!(request, ctx, {
        // Demonstrate both request APIs:
        // 1. RequestExt (direct from request) - available now
        // 2. ResourceParams (via ctx) - available after router injection

        // Option 1: Direct from request (always works)
        let request_id = request.id();
        let client_ip_from_req = request.ip().unwrap_or_else(|| "unknown".to_string());
        let hostname = request.hostname().unwrap_or_else(|| "unknown".to_string());

        // Option 2: From context (after router implements injection)
        let _request_id_from_ctx = ctx.request_id();
        let client_ip_from_ctx = ctx.client_ip();

        tracing::info!(
            "Metrics request: id={}, ip={} (from req), ip={} (from ctx), host={}",
            request_id,
            client_ip_from_req,
            client_ip_from_ctx.unwrap_or("not injected yet"),
            hostname
        );

        // Use tables() for Harper-compatible access
        let rules = ctx.get_table("Rule")?;
        let mut by_host: HashMap<String, usize> = HashMap::new();
        let mut by_status: HashMap<i64, usize> = HashMap::new();

        // Scan all records
        let records: Vec<serde_json::Value> = rules.get_all().await?;
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
                "clientIp": client_ip_from_req
            }
        }))
    });

    post!(request, ctx, {
        let request_id = request.id();
        let client_ip = request.ip().unwrap_or_else(|| "unknown".to_string());

        let rules = ctx.get_table("Rule")?;
        let records: Vec<serde_json::Value> = rules.get_all().await?;

        reply()
            .header("x-request-id", &request_id)
            .header("x-client-ip", &client_ip)
            .json(json!({
                "totalRules": records.len(),
                "requestId": request_id,
                "clientIp": client_ip,
                "apiStyle": "Fastify builder pattern"
            }))
    });

    put!(request, _ctx, {
        let request_id = request.id();

        reply()
            .header("x-request-id", &request_id)
            .messagepack(json!({
                "format": "messagepack",
                "efficient": true,
                "binary": true
            }))
    });

    patch!(request, _ctx, {
        let request_id = request.id();

        reply()
            .header("x-request-id", &request_id)
            .cbor(json!({
                "format": "cbor",
                "standard": "RFC 8949"
            }))
    });
}

register_resource!(RedirectMetrics);
