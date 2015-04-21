extern crate libc;

pub use ::{Application, ServurError};
use nickel::{Request, Response, MiddlewareResult};
use nickel::mimes::MediaType;
use nickel::status::StatusCode::{Accepted, BadRequest};
use std::ascii::AsciiExt;
use std::io::{self, Read};
use std::thread;
use std::error::Error;
use std::process::{ChildStdout, Command, Stdio};
use libc::funcs::posix88::signal::kill;
// use std::os::unix::prelude::ExitStatusExt;
use rustc_serialize::json;

const PIPE_BUF_SIZE: usize = 4096; // max stdout bytes read per interval
// const PIPE_INTERVAL: u64 = 500; // ms between checking stdout

#[derive(RustcEncodable)]
struct MessageResponse<'a> {
    message: &'a str
}

pub fn get_hello<'a>(_: &mut Request, response: Response<'a>)
        -> MiddlewareResult<'a> {
    response.send("Hello from Servur")
}

pub fn get_status<'a>(_: &mut Request, mut response: Response<'a>, app: &Application)
        -> MiddlewareResult<'a> {
    let status = app.read_status();
    response.content_type(MediaType::Json);
    response.send(json::encode(&status).unwrap())
}


pub fn post_signal<'a>(request: &mut Request, mut response: Response<'a>, app: &Application)
        -> MiddlewareResult<'a> {
    response.content_type(MediaType::Json);

    // Determine signal integer
    let signal_param = request.param("signal").to_ascii_uppercase();
    let message = match signal_from_str(&*signal_param) {
        Ok(signal) => {
            // Send the signal to the process
            // TODO: make sure the pid is set - currently the unwrap here panics if pid.is_none()
            let pid = app.pid.lock().unwrap().unwrap().clone();
            unsafe { kill(pid, signal) };
            response.set_status(Accepted);
            format!("Signaled '{}' with SIG{}", pid, signal_param)
        }
        Err(desc) => {
            println!("post_signal: {}", desc);
            response.set_status(BadRequest);
            format!("Error prevented signalling '{}': {}", app.runner, desc)
        }
    };

    let message_response = MessageResponse{message: &*message};
    response.send(json::encode(&message_response).unwrap())
}


pub fn post_run<'a>(request: &mut Request, response: Response<'a>, app: &Application)
        -> MiddlewareResult<'a> {
    let runner = &*(app.runner);
    let runner_args = &*(app.runner_args);
    // TODO: fail if busy running another process


    // Start child process
    let child = match Command::new(runner)
                        .args(runner_args)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn() {
        Err(why) => panic!("couldn't spawn {}: {}", runner, Error::description(&why)),
        Ok(process) => process,
    };

    // TODO: get the child's PID - waiting on stdlib support
    // app.set_pid(Some(child.id()));

    {
        // Pipe request body to child stdin
        match child.stdin {
            Some(mut stdin) => {
                io::copy(&mut request.origin, &mut stdin);
            },
            None => println!("No STDIN"),
        };
    }


    let tail = move |stdout: &mut ChildStdout| {
        loop {
            // TODO: Pipe both stdout and stderr:
            //       child.stdout | servur.stdout
            //       child.stderr | servur.sterr
            let mut buf = [0u8; PIPE_BUF_SIZE];
            match stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(_) => print!("{}", String::from_utf8_lossy(&buf)),
                Err(e) => {
                    print!("{}", String::from_utf8_lossy(&buf));
                    print!("{:?}", e);
                    break
                },

            };
        }
    };

    // pipe_child_output(&mut child);
    // match child.wait() {
    //     Err(..) => println!("TODO: what causes child.wait to have Err Result"),
    //     Ok(exit_condition) => match exit_condition.code() {
    //         Some(0) => println!("Finished without errors"),
    //         Some(a) => println!("Finished with error number: {}", a),
    //         None => println!("Terminated by signal number: {}", exit_condition.signal().unwrap()),
    //     }
    // };

    let mut stdout = child.stdout.unwrap();
    thread::spawn(move || tail(&mut stdout));

    let message = format!("Running {}", runner);
    let message_response = MessageResponse{message: &*message};
    response.send(json::encode(&message_response).unwrap())
}


//
// Private helper methods until further refactoring
//

fn signal_from_str(signal: &str) -> Result<i32, String> {
    match signal {
        // SIGTERM: terminate process (i.e. default kill)
        "TERM" => Ok(libc::SIGTERM),
        // SIGKILL: immediately kill process (i.e. kill -9)
        "KILL" => Ok(libc::SIGKILL),
        // SIGQUIT: quit and perform core dump
        "QUIT" => Ok(libc::consts::os::posix88::SIGQUIT),
        // Error for all other signals
        unknown => Err(format!("invalid signal: {}", unknown)),
    }
}

