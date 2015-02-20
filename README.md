# Servur Concept
A simple server to "serve [data to] your" tool or library. Servur starts a process, pipes data into it, monitors it, and can send signals to it - all from HTTP requests.

It's not particularly interesting alone, but it becomes interesting when you build a language/framework specific runner on top of this container.

[![Build Status](https://travis-ci.org/anowell/servur.svg)](https://travis-ci.org/anowell/servur)

## Servur runners
- [servur-ruby](https://github.com/anowell/servur-ruby)
- [servur-jar](https://github.com/anowell/servur-jar)

## Servur API

### GET /
Simple hello to verify servur is running.

#### Response:

    200 (OK)
    Hello from Servur

----------

### GET /status
Get's the status of Servur. The response includes the configured runner and it's args, and if the runner is running, it will respond with the runner's PID.

#### Response:

    200 (OK)
    {
        "runner": "wc",
        "runner_args": "-l"
        "pid": 1234
    }

----------

### POST /run
Posts any arbitrary body that will be passed into the runner as STDIN. Returns as soon as the runner has started.

TODO: The request should fail if the runner is already processing data.

#### Response:

    202 (OK)
    {
        "message": "Running 'wc'"
    }

----------

### POST /signal/:signal
Signals the runner (if running) with a specifix UNIX signal. Supports term, kill, and stop.

#### Response

    200 (OK)
    {
        "message": "Successfully signaled 'wc' with SIGKILL"
    }


## Usage
The docker image is published to [anowell/servur](https://registry.hub.docker.com/u/anowell/servur/) on the Docker Hub.

    $ docker run -p 8080:8080 anowell/servur
    $ curl -s localhost:8080/data -XPOST -d"
        An old silent pond...
        A frog jumps into the pond,
        splash! Silence again.
        "
    wc output:       4      13     102

## Building

    cargo build
    docker build .

