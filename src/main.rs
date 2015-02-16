#![feature(env)]
#![feature(io)]
#![feature(libc)]
#![feature(std_misc)]

extern crate nickel;
extern crate libc;
extern crate "rustc-serialize" as rustc_serialize;

use std::env;
use std::old_io;
use std::old_io::net::ip::Ipv4Addr;
use std::error::FromError;
use std::sync::{Arc,Mutex};
use nickel::router::http_router::HttpRouter;
use nickel::{Nickel, Request, Response};
// use rustc_serialize::json;
use libc::pid_t;

mod controller;

//
// Error Handling
//
#[derive(Debug)]
pub enum ArestError {
    IoError(old_io::IoError),
}

impl FromError<old_io::IoError> for ArestError {
    fn from_error(err: old_io::IoError) -> ArestError {
        ArestError::IoError(err)
    }
}


//
// Shared Application State
//
#[derive(Clone)]
struct Application {
    runner: String,
    pid: Arc<Mutex<Option<pid_t>>>,
}

#[derive(Clone, RustcEncodable)]
struct Status {
    runner: String,
    pid: Option<pid_t>
}

impl Application {
    fn set_pid(&self, pid: Option<pid_t>) {
        let mut shared_pid = self.pid.lock().unwrap();
        *shared_pid = pid;
    }

    fn read_status(&self) -> Status {
        Status {
            runner: self.runner.clone(),
            pid: self.pid.lock().unwrap().clone()
        }
    }
}


fn usage() {
    println!("Usage: arest PROGRAM");
    println!("  where PROGRAM is th executable to run on POST to /data");
    env::set_exit_status(1);
}

fn main() {
    let port = 8080;
    let runner: String = match env::args().nth(1) {
        Some(arg) => arg,
        None => { usage(); return; }
    };

    // Convenience types
    type ReqHandler = fn(&Request, &mut Response);
    type AppHandler = fn(&Request, &mut Response, &Application);

    // Initialize server and application state
    let mut server = Nickel::new();
    let app = Application{
        runner: runner,
        pid: Arc::new(Mutex::new(None)),
        // status: Arc::new(Mutex::new(Status{ pid: None, started: None })),
    };

    // Helpers to create routes: stateless (req_handler) and stateful (app_handler)
    let req_handler = |fn_ptr: ReqHandler| { fn_ptr };
    let app_handler = |fn_ptr: AppHandler| { (fn_ptr, app.clone()) };

    // App routing
    server.get("/",                     req_handler(controller::get_hello));
    server.get("/status",               app_handler(controller::get_status));
    server.post("/data",                app_handler(controller::post_data));
    server.post("/signal/:signal",      app_handler(controller::post_signal));

    // Start the server
    println!("Listening on port {}", port);
    server.listen(Ipv4Addr(127, 0, 0, 1), port);
}
