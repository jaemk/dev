use actix_web::{client, web, App, Error, HttpRequest, HttpResponse, HttpServer};

use crate::{CONFIG, LOG};

async fn status() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": CONFIG.version,
    })))
}

async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<client::Client>,
) -> Result<HttpResponse, Error> {
    let host = req.connection_info().host().to_string();
    let parts = host.split(&CONFIG.this_host_name).collect::<Vec<_>>();

    let (sub_domain, port) = if parts.len() == 2 {
        let sub_domain = if parts[0].is_empty() {
            None
        } else {
            Some(parts[0])
        };

        let port = if parts[1].is_empty() {
            80
        } else {
            parts[1]
                .trim_start_matches(':')
                .parse::<u16>()
                .expect("invaild port")
        };
        (sub_domain, port)
    } else if parts.len() == 1 {
        let part = parts[0];
        if part.contains(':') {
            let port = part
                .trim_start_matches(':')
                .parse::<u16>()
                .expect("invaild port");
            (None, port)
        } else {
            let part = if part.is_empty() { None } else { Some(part) };
            (part, 80)
        }
    } else {
        (None, 80)
    };
    slog::debug!(
        LOG,
        "host: {:?}, parts: {:?}, sub: {:?}, port: {:?}",
        host,
        parts,
        sub_domain,
        port
    );
    match sub_domain {
        None => {
            // proxy to a local dev server
            let qs = match req.uri().query() {
                Some(s) => format!("?{}", s),
                None => format!(""),
            };
            let new_url = format!(
                "http://localhost:{}{}{}",
                CONFIG.dev_server_port,
                req.uri().path(),
                qs
            );
            slog::info!(LOG, "proxying to dev server at: {}", new_url);
            let forwarded_req = client
                .request_from(new_url.as_str(), req.head())
                .no_decompress();
            let forwarded_req = if let Some(addr) = req.head().peer_addr {
                forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
            } else {
                forwarded_req
            };

            let mut res = forwarded_req.send_body(body).await.map_err(Error::from)?;

            let mut client_resp = HttpResponse::build(res.status());
            for (header_name, header_value) in
                res.headers().iter().filter(|(h, _)| *h != "connection")
            {
                client_resp.header(header_name.clone(), header_value.clone());
            }

            Ok(client_resp.body(res.body().await?))
        }
        Some(d) => match d {
            // handle subdomain redirects
            "status." => status().await,
            "git." => {
                let repo = req.uri().path();
                let redirect = format!("https://github.com/jaemk{}", repo);
                slog::info!(LOG, "redirecting to github repo {}", redirect);
                Ok(HttpResponse::TemporaryRedirect()
                    .header("Location", redirect)
                    .finish())
            }
            _ => Ok(HttpResponse::NotFound().body("nothing to see here")),
        },
    }
}

pub async fn start() -> anyhow::Result<()> {
    let addr = format!("{}:{}", CONFIG.host, CONFIG.port);
    slog::info!(LOG, "** Listening on {} **", addr);

    HttpServer::new(|| {
        App::new()
            .data(client::Client::new())
            .wrap(crate::logger::Logger::new())
            .default_service(web::route().to(forward))
    })
    .bind(addr)?
    .run()
    .await?;
    Ok(())
}
