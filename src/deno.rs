use actix_rt::task::yield_now;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::str;
use std::sync::atomic::Ordering;
use std::time::Instant;
use uuid::Uuid;

//static BIN_DENO: &str = "/home/XXX/.deno/bin/deno";
use crate::GLOBAL_DENO_THREAD_COUNT;
use crate::MAX_DENO_THREADS;

//
//
//
async fn write(filename: &str, code: &str) -> bool {
    if code.is_empty() {
        return false;
    };

    let mut fd;
    match File::create(filename.to_string()) {
        Ok(v) => fd = v,
        Err(_) => {
            println!("Failed to create {}", filename);
            return false;
        }
    };

    match fd.write_all(code.as_bytes()) {
        Ok(_) => return true,
        _ => return false,
    };
}

pub fn remove(filename: &str) {
    match fs::remove_file(&filename) {
        Ok(_) => (),
        Err(_) => (),
    }
}

pub async fn run(code: &str) -> HttpResponse {
    println!("DenoRun-->{:?}", code);

    let deno_binary = match env::var("DENO_BIN") {
        Ok(val) => val,
        Err(_) => {
            return HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .body("DENO_BIN not set as environment variable.".to_string());
        }
    };

    if code.contains("InvocaMiddleware") == false {
        return HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .body("Incorrect secret.".to_string());
    }

    let start_time = Instant::now();

    let filename = format!("/tmp/{}", Uuid::new_v4().to_string());

    if write(&filename, code).await == false {
        return HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .body("Failed to process code.".to_string());
    }

    // This thread count is for the deno children, not the Tokio threads
    loop {
        let children_count = GLOBAL_DENO_THREAD_COUNT.fetch_add(0, Ordering::SeqCst);
        if children_count < MAX_DENO_THREADS {
            break;
        }
        yield_now().await;
    }

    // We only allow network access for the deno threads
    let mut child;
    match Command::new(deno_binary)
        .arg("run")
        .arg(format!("--allow-read={}", filename))
        .arg("--allow-net")
        .arg(&filename)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(v) => {
            GLOBAL_DENO_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
            child = v;
        }
        Err(e) => {
            println!("Got error spawning deno {:?}", e);
            remove(&filename);
            return HttpResponse::Ok()
                .status(StatusCode::EXPECTATION_FAILED)
                .body("Failed to start service.".to_string());
        }
    };

    //
    // Wait for the child to end, letting other Tokio threads run
    //
    let status_string;

    loop {
        match child.try_wait() {
            Ok(Some(v)) => {
                status_string = format!("{}", v);
                break;
            }
            Ok(None) => (),
            Err(e) => {
                status_string = format!("{}", e);
                break;
            }
        };
        yield_now().await;
    }
    GLOBAL_DENO_THREAD_COUNT.fetch_add(-1, Ordering::SeqCst);
    remove(&filename);

    let duration = start_time.elapsed().as_millis() as u64;
    if status_string == "exit status: 0" {
        let message = format!("Success running deno code in {} ms.", duration);
        return HttpResponse::Ok().status(StatusCode::OK).body(message);
    } else {
        let message = format!("Failed running deno code in {} ms.", duration);
        return HttpResponse::Ok()
            .status(StatusCode::EXPECTATION_FAILED)
            .body(message);
    }
}
