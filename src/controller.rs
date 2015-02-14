extern crate http;
extern crate libc;

use ::{Application, ArestError};
use nickel::{Request, Response};
use std::thread::Thread;
use std::old_io::process::{Process, Command, ProcessExit};

pub fn get_hello(_: &Request, response: &mut Response) {
    response.send("Alive and well");
}

pub fn get_status(_: &Request, response: &mut Response, app: &Application) {
    let runner = &*(app.runner);
    let pid = app.pid.lock().unwrap().unwrap_or(0);

    println!("get_status: runner({:?}) pid({:?})", runner, pid);
    response.send(format!("Arest runner: {}", runner));
}

pub fn post_signal(request: &Request, response: &mut Response, app: &Application) {
    let signal_param = request.param("signal");
    let signal = match signal_param {
        // SIGTERM: terminate process (i.e. default kill)
        "term" => libc::SIGTERM as isize,
        // SIGKILL: immediately kill process (i.e. kill -9)
        "kill" => libc::SIGKILL as isize,
        // SIGQUIT: quit and perform core dump
        "quit" => libc::consts::os::posix88::SIGQUIT as isize,
        // Error for all other signals
        unknown_signal => {
            println!("post_signal: invalid signal: {}", unknown_signal);
            response.status_code(http::status::BadRequest);
            response.send(format!("post_signal: invalid signal: {}", unknown_signal));
            return;
        }
    };

    // TODO: make sure the pid is set
    let pid = app.pid.lock().unwrap().unwrap().clone();
    match Process::kill(pid, signal) {
        Ok(_) => {
            response.send(format!("Successfully signaled {} with {}", pid, signal_param));
        },
        Err(desc) => {
            response.send(format!("Error signaling {} with {}: {:?}", pid, signal_param, desc));
        }
    };
}

const PIPE_BUF_SIZE: usize = 4096; // max bytes read per interval
const PIPE_INTERVAL: u64 = 500; // ms between checking stdout

fn pipe_child_output(child: &mut Process) {
      let mut buf = [0u8; PIPE_BUF_SIZE];
      child.stdout.as_mut().unwrap().read(&mut buf).ok();
      // TODO: Pipe both stdout and stderr:
      //       child.stdout | arest.stdout
      //       child.stderr | arest.sterr
      print!("{}", String::from_utf8_lossy(&buf));
}

pub fn post_data(request: &Request, response: &mut Response, app: &Application) {
    let runner = &*(app.runner);
    // TODO: fail if busy running another process

    let spawn_runner = || -> Result<Process, ArestError> {
        // Start child process
        let mut child = try!(Command::new(runner).spawn());
        println!("Started {} with pid: {}", runner, child.id());
        app.set_pid(Some(child.id()));

        // Pipe request body to child stdin
        let mut child_stdin = child.stdin.take().unwrap();
        try!(child_stdin.write_all(&request.origin.body));
        Ok(child)
    };

    match spawn_runner() {
        Err(why) => {
            response.status_code(http::status::InternalServerError);
            let message = format!("Could not read {} stdout: {:?}", runner, why);
            println!("{}", message);
            response.send(message);
        },
        Ok(mut child) =>  {
            response.status_code(http::status::Accepted);
            Thread::spawn(move || {
                loop {
                    // This is how we stream the stdout pipe: a chunk every PIPE_INTERVAL
                    child.set_timeout(Some(PIPE_INTERVAL));
                    match child.wait() {
                        Err(..) => {
                            // Err is generally TimedOut or EndOfFile
                            //   but regardless, we pipe until a ProcessExit happens
                            pipe_child_output(&mut child);
                        }
                        Ok(exit_condition) => {
                            pipe_child_output(&mut child);
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
            });
        },
    };







}
