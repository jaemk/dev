use actix_files::Files;
use actix_web::{body, client, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use rustls::{NoClientAuth, ServerConfig};

use crate::{CONFIG, LOG};

async fn status() -> actix_web::Result<HttpResponse> {
    let client = client::Client::default();

    let badge = client
        .get("https://badge-cache.kominick.com/status")
        .send()
        .await
        .map(|resp| resp.status().as_u16())
        .unwrap_or(500);
    let kom = client
        .get("https://kominick.com/status")
        .send()
        .await
        .map(|resp| resp.status().as_u16())
        .unwrap_or(500);
    let james = client
        .get("https://james.kominick.com/status")
        .send()
        .await
        .map(|resp| resp.status().as_u16())
        .unwrap_or(500);
    let paste = client
        .get("https://paste.kominick.com/status")
        .send()
        .await
        .map(|resp| resp.status().as_u16())
        .unwrap_or(500);
    let transfer = client
        .get("https://transfer.kominick.com/status")
        .send()
        .await
        .map(|resp| resp.status().as_u16())
        .unwrap_or(500);
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": CONFIG.version,
        "badge_status": badge,
        "kom_status": kom,
        "james_status": james,
        "paste_status": paste,
        "transfer_status": transfer,
    })))
}

async fn forward(
    req: HttpRequest,
    body: web::Payload,
    client: web::Data<client::Client>,
) -> Result<HttpResponse, Error> {
    let host = req.connection_info().host().to_string();
    slog::info!(LOG, "forwarding for host: {}", host,);
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
            let port = parts[1].trim_start_matches(':');
            port.parse::<u16>()
                .map_err(|e| anyhow::anyhow!("invalid port {}, {}", port, e))
                .expect("invaild port")
        };
        (sub_domain, port)
    } else if parts.len() == 1 {
        let part = parts[0];
        if part.contains(':') {
            let port = part.trim_start_matches(':');
            let port = port
                .parse::<u16>()
                .map_err(|e| anyhow::anyhow!("invalid port {}, {}", port, e))
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

            let mut res = forwarded_req
                .send_body(body::BodyStream::new(Box::new(body)))
                .await
                .map_err(Error::from)?;

            slog::info!(LOG, "proxy requests got back status {}", res.status());
            let mut client_resp = HttpResponse::build(res.status());
            for (header_name, header_value) in
                res.headers().iter().filter(|(h, _)| *h != "connection")
            {
                client_resp.header(header_name.clone(), header_value.clone());
            }

            // don't proxy the response if it's not-modified
            if res.status() == 304 {
                Ok(client_resp.finish())
            } else {
                Ok(client_resp.body(res.body().await?))
            }
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
            "www." => {
                slog::info!(LOG, "redirecting www. to jaemk.me");
                Ok(HttpResponse::TemporaryRedirect()
                    .header("Location", "https://jaemk.me")
                    .finish())
            }
            _ => Ok(HttpResponse::NotFound().body("nothing to see here")),
        },
    }
}

pub async fn start() -> anyhow::Result<()> {
    let addr = format!("{}:{}", CONFIG.host, CONFIG.port);
    let ssl_addr = format!("{}:{}", CONFIG.host, CONFIG.ssl_port);
    slog::info!(LOG, "** Listening on {} **", addr);

    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut std::io::BufReader::new(
        std::fs::File::open("bin/cert.pem").expect("failed to open cert"),
    );
    let key_file = &mut std::io::BufReader::new(
        std::fs::File::open("bin/key.pem").expect("failed to open key"),
    );
    let cert_chain = certs(cert_file).expect("failed to load cert");
    let mut keys = pkcs8_private_keys(key_file).expect("failed to load key");
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    let serve_ssl = || async move {
        HttpServer::new(|| {
            App::new()
                .data(client::Client::new())
                .wrap(crate::logger::Logger::new())
                .service(web::resource("/.status").route(web::get().to(status)))
                .service(Files::new("/.static", "static"))
                .service(Files::new("/.well-known", "static/.well-known"))
                .default_service(web::route().to(forward))
        })
        .bind_rustls(&ssl_addr, config)
        .unwrap_or_else(|e| panic!("failed to bind to {} {:?}", ssl_addr, e))
        .run()
        .await
        .expect("error running ssl server")
    };
    actix_web::rt::spawn(serve_ssl());

    HttpServer::new(|| {
        App::new()
            .data(client::Client::new())
            .wrap(crate::logger::Logger::new())
            .service(web::resource("/.status").route(web::get().to(status)))
            .service(Files::new("/.static", "static"))
            .service(Files::new("/.well-known", "static/.well-known"))
            .default_service(web::route().to(forward))
    })
    .bind(&addr)
    .unwrap_or_else(|e| panic!("failed to bind to {} {:?}", addr, e))
    .run()
    .await
    .expect("error running server");

    Ok(())
}
