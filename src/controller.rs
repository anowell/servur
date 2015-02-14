extern crate http;
extern crate libc;

use ::{Application, ArestError};
use std::old_io::process::{Process, Command};
use nickel::{Request, Response};

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
        // SIGQUIT: quit and perform core dump
        "sigquit" => libc::consts::os::posix88::SIGQUIT as isize,
        // SIGKILL: immediately kill process (i.e. kill -9)
        "sigkill" => libc::SIGKILL as isize,
        // SIGTERM: terminate process (i.e. default kill)
        "sigterm" => libc::SIGTERM as isize,
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

pub fn post_data(request: &Request, response: &mut Response, app: &Application) {
    let runner = &*(app.runner);

    let spawn_runner = || {
        let mut process = try!(Command::new(runner).spawn());
        app.set_pid(Some(process.id()));
        let mut child_stdin = process.stdin.take().unwrap();
        try!(child_stdin.write_all(&request.origin.body));
        Ok(process)
    };

    let process_result: Result<Process, ArestError> = spawn_runner();
    let message = match process_result {
        Ok(mut process) =>  {
            let output = process.stdout.as_mut().unwrap().read_to_string().ok().unwrap();
            format!("{} output: {}!", runner, output)
        },
        Err(why) => {
            response.status_code(http::status::InternalServerError);
            format!("Could not read {} stdout: {:?}", runner, why)
        }
    };

    println!("{}", message);
    response.send(message);
}
