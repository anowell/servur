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
use libc::pid_t;

mod controller;

//
// Error Handling
//
#[derive(Debug)]
pub enum ServurError {
    IoError(old_io::IoError),
}

impl FromError<old_io::IoError> for ServurError {
    fn from_error(err: old_io::IoError) -> ServurError {
        ServurError::IoError(err)
    }
}


//
// Shared Application State
//
#[derive(Clone)]
struct Application {
    runner: String,
    runner_args: Vec<String>,
    pid: Arc<Mutex<Option<pid_t>>>,
}

#[derive(Clone, RustcEncodable)]
struct Status {
    runner: String,
    runner_args: Vec<String>,
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
            runner_args: self.runner_args.clone(),
            pid: self.pid.lock().unwrap().clone()
        }
    }
}


fn usage() {
    println!("Usage: servur PROGRAM [PROGRAM_ARGS]");
    println!("  where PROGRAM is the executable to run on POST to /run");
    env::set_exit_status(1);
}

fn main() {
    let port = 8080;

    // discard args.nth(0), args.nth(1) is the runner
    // the remaining args are passed to the runner
    let mut args = env::args();
    args.next();
    let (runner, runner_args): (String, Vec<String>) = match args.next() {
        Some(ref arg) if *arg == "-h" || *arg == "--help" => { usage(); return; },
        Some(arg) => (arg, args.collect()),
        None => { usage(); return; }
    };

    // Convenience types
    type ReqHandler = fn(&Request, &mut Response);
    type AppHandler = fn(&Request, &mut Response, &Application);

    // Initialize server and application state
    let mut server = Nickel::new();
    let app = Application{
        runner: runner,
        runner_args: runner_args,
        pid: Arc::new(Mutex::new(None)),
    };

    // Helpers to create routes: stateless (req_handler) and stateful (app_handler)
    let req_handler = |fn_ptr: ReqHandler| { fn_ptr };
    let app_handler = |fn_ptr: AppHandler| { (fn_ptr, app.clone()) };

    // App routing
    server.get("/",                     req_handler(controller::get_hello));
    server.get("/status",               app_handler(controller::get_status));
    server.post("/run",                 app_handler(controller::post_run));
    server.post("/signal/:signal",      app_handler(controller::post_signal));

    // Start the server
    println!("Listening on port {}", port);
    server.listen(Ipv4Addr(0, 0, 0, 0), port);
}
