#![feature(libc)]

extern crate nickel;
extern crate libc;
extern crate rustc_serialize;

use std::io;
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc,Mutex};
use nickel::router::http_router::HttpRouter;
use nickel::{Nickel, Request, Response, MiddlewareResult};
use libc::pid_t;

mod controller;

//
// Error Handling
//
#[derive(Debug)]
pub enum ServurError {
    Generic(String),
    IoError(io::Error),
}

impl From<io::Error> for ServurError {
    fn from(err: io::Error) -> ServurError {
        ServurError::IoError(err)
    }
}


//
// Shared Application State
//
#[derive(Clone)]
pub struct Application {
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
    std::process::exit(1);
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
    type ReqHandler = for <'a> fn(&mut Request, Response<'a>) -> MiddlewareResult<'a>;
    type AppHandler = for <'a> fn(&mut Request, Response<'a>, &Application) -> MiddlewareResult<'a>;

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
    server.listen(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port));
}