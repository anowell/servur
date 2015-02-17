extern crate http;
extern crate libc;

use ::{Application, ServurError};
use nickel::{Request, Response};
use nickel::mimes::MediaType;
use std::thread::Thread;
use std::old_io::process::{Process, Command, ProcessExit};
use rustc_serialize::json;

const PIPE_BUF_SIZE: usize = 4096; // max bytes read per interval
const PIPE_INTERVAL: u64 = 500; // ms between checking stdout

#[derive(RustcEncodable)]
struct MessageResponse<'a> {
    message: &'a str
}

pub fn get_hello(_: &Request, response: &mut Response) {
    response.send("Hello from Servur");
}

pub fn get_status(_: &Request, response: &mut Response, app: &Application) {
    let status = app.read_status();
    response.content_type(MediaType::Json);
    response.send(json::encode(&status).unwrap());
}


pub fn post_signal(request: &Request, response: &mut Response, app: &Application) {
    response.content_type(MediaType::Json);

    // Determine signal integer
    let signal_param = request.param("signal");
    let message = match signal_from_str(signal_param) {
        Ok(signal) => {
            // Send the signal to the process
            // TODO: make sure the pid is set - currently the unwrap here panics if pid.is_none()
            let pid = app.pid.lock().unwrap().unwrap().clone();
            match Process::kill(pid, signal) {
                Ok(_) => {
                    response.status_code(http::status::Accepted);
                    format!("Successfully signaled {} with {}", pid, signal_param)
                },
                Err(desc) => {
                    println!("post_signal: {}", desc);
                    response.status_code(http::status::InternalServerError);
                    format!("Error signaling {} with {}: {:?}", pid, signal_param, desc)
                }
            }
        }
        Err(desc) => {
            println!("post_signal: {}", desc);
            response.status_code(http::status::BadRequest);
            format!("Error prevented signalling {}: {}", app.runner, desc)
        }
    };

    let message_response = MessageResponse{message: &*message};
    response.send(json::encode(&message_response).unwrap());
}


pub fn post_data(request: &Request, response: &mut Response, app: &Application) {
    let runner = &*(app.runner);
    let runner_args = &*(app.runner_args);
    // TODO: fail if busy running another process

    //
    // Closure to spawn the runner child
    //
    let spawn_runner = || -> Result<Process, ServurError> {
        // Start child process
        let mut child = try!(Command::new(runner).args(runner_args).spawn());
        println!("Started {} with pid: {}", runner, child.id());
        app.set_pid(Some(child.id()));

        // Pipe request body to child stdin
        let mut child_stdin = child.stdin.take().unwrap();
        try!(child_stdin.write_all(&request.origin.body));
        Ok(child)
    };

    //
    // Closure to wait on runner child while tailing stdout
    //
    let wait_with_tail = move |child: &mut Process| {
        loop {
            // This is how we stream the stdout pipe: a chunk every PIPE_INTERVAL
            child.set_timeout(Some(PIPE_INTERVAL));
            match child.wait() {
                Err(..) => {
                    // Err is generally TimedOut or EndOfFile
                    //   but regardless, we pipe until a ProcessExit happens
                    pipe_child_output(child);
                }
                Ok(exit_condition) => {
                    pipe_child_output(child);
                    match exit_condition {
                        ProcessExit::ExitStatus(0) => println!("Finished without errors"),
                        ProcessExit::ExitStatus(a) => println!("Finished with error number: {}", a),
                        ProcessExit::ExitSignal(a) => println!("Terminated by signal number: {}", a),
                    }
                    // TODO: update Application state
                    return;
                }
            }
        }
    };

    //
    // Controller logic to spawn runner child and then tail it
    //   without blocking
    //
    let message = match spawn_runner() {
        Err(why) => {
            response.status_code(http::status::InternalServerError);
            println!("post_data:: failed to spawn_runner: {:?}", why);
            format!("Error running {}: {:?}", runner, why)
        },
        Ok(mut child) => {
            response.status_code(http::status::Accepted);
            Thread::spawn(move || wait_with_tail(&mut child));
            format!("Running {}", runner)
        },
    };

    let message_response = MessageResponse{message: &*message};
    response.send(json::encode(&message_response).unwrap());
}


//
// Private helper methods until further refactoring
//

fn signal_from_str(signal: &str) -> Result<isize, String> {
    match signal {
        // SIGTERM: terminate process (i.e. default kill)
        "term" => Ok(libc::SIGTERM as isize),
        // SIGKILL: immediately kill process (i.e. kill -9)
        "kill" => Ok(libc::SIGKILL as isize),
        // SIGQUIT: quit and perform core dump
        "quit" => Ok(libc::consts::os::posix88::SIGQUIT as isize),
        // Error for all other signals
        unknown => Err(format!("invalid signal: {}", unknown)),
    }
}

fn pipe_child_output(child: &mut Process) {
      let mut buf = [0u8; PIPE_BUF_SIZE];
      child.stdout.as_mut().unwrap().read(&mut buf).ok();
      // TODO: Pipe both stdout and stderr:
      //       child.stdout | servur.stdout
      //       child.stderr | servur.sterr
      print!("{}", String::from_utf8_lossy(&buf));
}
