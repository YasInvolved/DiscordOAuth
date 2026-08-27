use std::net::SocketAddr;
use axum::http::HeaderMap;

pub fn log_endpoint(method: &str, endpoint: &str, addr: SocketAddr, headers: HeaderMap) {
    let method_cap = method.to_ascii_uppercase();

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");

    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| addr.ip().to_string());

    println!("[{method_cap}] {endpoint} | IP: {client_ip} | User-Agent: {user_agent}");
}