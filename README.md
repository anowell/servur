# Servur Concept
A simple server to "serve [data to] your" tool or library. Servur starts a process, pipes data into it, monitors it, and can send signals to it - all from HTTP requests.

It's not particularly interesting alone, but it becomes interesting when you build a language/framework specific runner on top of this container.

[![Build Status](https://travis-ci.org/anowell/servur.svg)](https://travis-ci.org/anowell/servur)

## Servur runners
- [servur-ruby](https://github.com/anowell/servur-ruby)
- [servur-jar](https://github.com/anowell/servur-jar)

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

