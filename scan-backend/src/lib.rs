//! Vetted scan backend — hello scaffold. Step 4 replaces the route table with
//! the real verdict engine; the shape stays: pure route function (unit-testable
//! on any target) + thin `#[event(fetch)]` wrapper + permissive CORS (knowledge
//! workers-rust.md — scans are anonymous, no session state).

use worker::*;

/// Pure route logic: (method, path) -> (status, content-type, body).
/// Host-independent on purpose — CI runs this on the native target.
pub fn route(method: &str, path: &str) -> (u16, &'static str, &'static str) {
    match path {
        "/health" => match method {
            "GET" => (200, "application/json", r#"{"ok":true}"#),
            _ => (
                405,
                "application/json",
                r#"{"error":"method_not_allowed"}"#,
            ),
        },
        _ => (404, "application/json", r#"{"error":"not_found"}"#),
    }
}

fn json_headers() -> Result<Headers> {
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    Ok(headers)
}

#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let (status, _, body) = route(&req.method().to_string(), &req.path());
    Ok(Response::ok(body)?
        .with_status(status)
        .with_headers(json_headers()?))
}

#[cfg(test)]
mod tests {
    use super::route;

    #[test]
    fn health_is_ok_json() {
        let (status, content_type, body) = route("GET", "/health");
        assert_eq!(status, 200);
        assert_eq!(content_type, "application/json");
        assert_eq!(body, r#"{"ok":true}"#);
    }

    #[test]
    fn wrong_method_is_405() {
        assert_eq!(route("POST", "/health").0, 405);
    }

    #[test]
    fn unknown_path_is_404() {
        assert_eq!(route("GET", "/nope").0, 404);
    }
}
