#![feature(plugin)]
#![feature(io)]

#[plugin]
#[macro_use]
#[no_link]
extern crate rustful_macros;

extern crate rustful;

use std::env;
use std::error::Error;
use std::old_io::process::Command;

use rustful::{Server, Context, Response, TreeRouter};
use rustful::Method::{Get, Post};
use rustful::StatusCode::{InternalServerError, BadRequest};

fn get_hello(_: Context, response: Response) {
    try_send!(
        response.into_writer(),
        "Arest is alive and well!",
        "calling get_hello"
    );
}

fn post_data(context: Context, mut response: Response) {
    let runner = match env::args().nth(1) {
        Some(arg) => arg.into_string().unwrap(),
        None => "wc".to_string(),
    };
    // let runner = if args.len() > 1 { &*args[1] } else { "wc" };

    let mut body_reader = context.body_reader;
    let data = match body_reader.read_to_end() {
        Ok(body) => body,
        Err(why) => {
            response.set_status(BadRequest);
            try_send!(response.into_writer(), format!("couldn't read request body: {}", why.description()));
            return;
        }
    };

    let mut process = match Command::new(&*runner).spawn() {
        Ok(process) => process,
        Err(why) => {
            response.set_status(InternalServerError);
            try_send!(response.into_writer(), format!("Could not spawn {}: {}", runner, why.description()));
            return;
        }
    };

    {
        // Would be nice to have method like std::io::copy for Reader/Writer
        //   std::io::copy(&mut body_reader, &mut child_stdout)
        //   Really, std:io needs to stabilize so we can move away from std:old_io
        let mut child_stdin = process.stdin.take().unwrap();
        match child_stdin.write_all(&*data) {
            Ok(_) => println!("Sending input data to {}...", runner),
            Err(why) => {
                response.set_status(InternalServerError);
                try_send!(response.into_writer(), format!("Could not write to {} stdin: {}", runner, why.description()));
                return;
            }
        }
    }

    // Let's do something with stdout
    match process.stdout.as_mut().unwrap().read_to_string() {
        Ok(output) => {
            println!("{} responded with:\n{}", runner, output);
            try_send!(response.into_writer(), format!("{} output: {}!", runner, output), "reading stdout");
        },
        Err(why) => {
            response.set_status(InternalServerError);
            try_send!(response.into_writer(), format!("Could not read {} stdout: {}", runner, why.description()));
            return;
        }
    }

}

fn main() {
    let router = TreeRouter::from_routes(
        vec![
            // The clumsy fn ptr cast is waiting on a compiler fix to address fn coersion
            (Get,   "/",        get_hello   as fn(Context, Response)),
            (Post,  "/data",    post_data   as fn(Context, Response)),
            // (Get,   "/status",  get_status  as fn(Context, Response)),
            // (Post,  "/interrupt",            post_interrupt   as fn(Context, Response)),
            // (Post,  "/interrupt/:signal",    post_interrupt   as fn(Context, Response)),
        ]
    );

    let port = 8080;
    let server_result = Server::new().port(port).handlers(router).run();

    match server_result {
        Ok(_server) => println!("Listening on port {}", port),
        Err(e) => println!("Could not start server: {}", e.description())
    }
}
