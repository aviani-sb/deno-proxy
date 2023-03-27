use actix_web::http::StatusCode;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use std::fs::read;
use std::process;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use systemstat::{Platform, System};
use uuid::Uuid;

mod deno;
mod utils;

static GLOBAL_DENO_THREAD_COUNT: AtomicIsize = AtomicIsize::new(0);
static MAX_DENO_THREADS: isize = 50;
static GLOBAL_REQUEST_COUNT: AtomicUsize = AtomicUsize::new(0);
static VERSION: &str = "DENO-HELPER.01";

#[get("/")]
async fn about() -> impl Responder {
    send_file("about.html", "text/html")
}

//
//
//
#[get("/stats")]
async fn stats() -> impl Responder {
    println!("Got requests for stats");
    let req_count = GLOBAL_REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
    let deno_thread_count = GLOBAL_DENO_THREAD_COUNT.fetch_add(0, Ordering::SeqCst);

    let sys = System::new();

    let mut payload = json::JsonValue::new_object();

    //
    // Get the system load averages
    //
    match sys.load_average() {
        Ok(loadavg) => {
            let value = format!(
                "1m:{} 5m:{} 15m:{}",
                loadavg.one, loadavg.five, loadavg.fifteen
            );
            payload = utils::add_tv(payload, "load_average", &value);
        }
        Err(_) => (),
    }

    let value = Uuid::new_v4().to_string();
    payload = utils::add_tv(payload, "version", VERSION);
    payload = utils::add_tv(payload, "pid", &process::id().to_string());
    payload = utils::add_tv(payload, "unique_id", &value);
    payload = utils::add_tv(payload, "total_requests", &req_count.to_string());
    payload = utils::add_utv(payload, "unix_epoch", utils::get_unix_epoch());
    payload = utils::add_tv(payload, "deno_thread_count", &deno_thread_count.to_string());

    HttpResponse::Ok().body(payload.dump())
}

//
//
//
fn send_file(path: &str, mime: &str) -> HttpResponse {
    let full_path = match std::env::current_dir() {
        Ok(val) => format!("{}/static/{}", val.display(), path),
        Err(_) => format!("/tmp/{}", path),
    };

    match read(&full_path) {
        Ok(v) => {
            return HttpResponse::Ok()
                .append_header(("Content-Type", mime))
                .body(v)
        }
        Err(e) => {
            println!("Failed to open {} {}", &full_path, e);
            return HttpResponse::Ok()
                .status(StatusCode::EXPECTATION_FAILED)
                .body("");
        }
    };
}

//
// The core
//
#[post("/deno")]
async fn deno_run(req_body: String) -> HttpResponse {
    GLOBAL_REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Got deno request, so sending it!");
    return deno::run(&req_body).await;
}

//
//
//
//
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!(
        "At {} started Deno Proxy process #{} on port 9999",
        utils::get_unix_epoch(),
        process::id()
    );

    //
    // This is our primary service
    //
    let _s = HttpServer::new(|| App::new().service(stats).service(about).service(deno_run))
        .bind("0.0.0.0:9999")
        .unwrap()
        .run()
        .await;

    Ok(())
}
