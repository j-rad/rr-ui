use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct DecoyMiddleware {
    pub secret_path: String,
    pub decoy_path: Option<String>,
}

impl DecoyMiddleware {
    pub fn new(secret_path: String, decoy_path: Option<String>) -> Self {
        Self {
            secret_path,
            decoy_path,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for DecoyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = DecoyMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DecoyMiddlewareService {
            service: Rc::new(service),
            secret_path: Rc::new(self.secret_path.clone()),
            decoy_path: Rc::new(self.decoy_path.clone()),
        })
    }
}

pub struct DecoyMiddlewareService<S> {
    service: Rc<S>,
    secret_path: Rc<String>,
    decoy_path: Rc<Option<String>>,
}

impl<S, B> Service<ServiceRequest> for DecoyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let secret_path = self.secret_path.clone();
        let decoy_path = self.decoy_path.clone();
        let service = self.service.clone();

        // Check if path starts with secret_path OR is an API call (if we want to hide API too, user must set secret path correctly)
        // Assumption: /panel/api is the structure. If secret_path is /panel, then /panel/api matches.
        // What about assets? If assets are at root /assets, they might be blocked.
        // User instruction: "If the request path does NOT start with 'panel_secret_path', serve static HTML files from 'decoy_site_path'."
        // This effectively blocks root / and anything else.
        // We should ensure strict adherence.

        // Check if path starts with secret_path OR is an API call
        // Also whitelist static assets path /_app/ and favicon
        if path.starts_with(secret_path.as_str())
            || path.starts_with("/_app/")
            || path == "/favicon.ico"
        {
            return Box::pin(async move {
                service.call(req).await.map(|res| res.map_into_boxed_body())
            });
        }

        // Serve decoy
        Box::pin(async move {
            if let Some(decoy_dir) = decoy_path.as_ref() {
                // Try to serve index.html from decoy_dir
                // This is a simplified static file server for just the index or request path?
                // Usually simple decoy means serving index.html for everything or matching path.
                // Let's implement serving index.html for root, and try file for others?
                // Instruction says: "static HTML files from 'decoy_site_path'".
                // So we treat decoy_site_path as a root dir.

                let safe_path = path.trim_start_matches('/');
                let file_path = if safe_path.is_empty() {
                    format!("{}/index.html", decoy_dir)
                } else {
                    format!("{}/{}", decoy_dir, safe_path)
                };

                if let Ok(mut file) = File::open(&file_path).await {
                    let mut contents = Vec::new();
                    if file.read_to_end(&mut contents).await.is_ok() {
                        let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
                        // We need to return a ServiceResponse.
                        // But B type is generic. Constructing ServiceResponse<B> manually is hard if B is not Body.
                        // Usually simpler to return HttpResponse and map it?
                        // But we are in middleware.
                        // Hack: We can use `req.into_response(HttpResponse)` structure.
                        let res = HttpResponse::Ok()
                            .content_type(mime.as_ref())
                            .body(contents);

                        return Ok(req.into_response(res));
                        // This is tricky without knowing B.
                        // Standard actix middleware pattern for intercepting and returning response
                        // usually requires B to be compatible with HttpResponse body or use Either.
                        // But here B is generic.
                        // If we error, we return Error.
                        // If we succeed with non-service response, we are "terminating" the chain.
                        // For terminating middleware, we usually need S to equal something compatible.
                        // Actually, common practice is:
                        // Middleware returns Result<ServiceResponse<B>, Error>.
                        // To return a custom response, we need to ensure it matches B.
                        // If we cannot guarantee B, we might be stuck.
                        // However, typically B=BoxBody or similar in main App.

                        // Alternative:: Use `actix_web::middleware::from_fn` or similar which handles this?
                        // Or just return ErrorForbidden/NotFound which Actix turns into response?
                        // But we want 200 OK decoy content.
                    }
                }
            }

            // If decoy not found or not set, return generic Nginx welcome page
            let body = r#"<!DOCTYPE html>
<html>
<head>
<title>Welcome to nginx!</title>
<style>
html { color-scheme: light dark; }
body { width: 35em; margin: 0 auto;
font-family: Tahoma, Verdana, Arial, sans-serif; }
</style>
</head>
<body>
<h1>Welcome to nginx!</h1>
<p>If you see this page, the nginx web server is successfully installed and
working. Further configuration is required.</p>

<p>For online documentation and support please refer to
<a href="http://nginx.org/">nginx.org</a>.<br/>
Commercial support is available at
<a href="http://nginx.com/">nginx.com</a>.</p>

<p><em>Thank you for using nginx.</em></p>
</body>
</html>"#;
            // Return "fake" 404 or just 200 OK?
            // Usually generic page is 200 OK.
            // We return Error which renders to HTTP Response.
            // But we want valid HTML.
            // Using `Err(actix_web::error::ErrorNotFound("..."))` leads to default error handler.
            // We can create a custom error that renders this HTML?
            Ok(req.into_response(HttpResponse::Ok().content_type("text/html").body(body)))
            // Wait, `into_response` expects `B` matches. `HttpResponse` has `BoxBody` usually.
            // If `B` is not `BoxBody`, this fails compilation.
            // In `lib.rs`: `App::new()...`. The default body type is `BoxBody`.
            // So `ServiceResponse<BoxBody>` (or similar) is expected.
            // But here `B` is generic.
            // To support generic `B`, we have to cast.
            // Usually middleware is used with `ServiceResponse<BoxBody>`.
            // Let's assume B is compatible `actix_web::body::BoxBody`.
            // We need `map_into_boxed_body` or similar.
            // Check Actix docs: `req.into_response(res)` creates `ServiceResponse`.
            // But `res` body type must match `B` or we map it.
            // If we change `B` to `actix_web::body::BoxBody` in the middleware definition, it restricts usage.
        })
    }
}
