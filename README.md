# Arest Proof-of-Concept
A simple RESTish server for running other processes

It's not particularly interesting alone, rather intended to be built on top of for language/framework specific implementations (e.g. [arest-ruby](https://github.com/anowell/arest-ruby))

[![Build Status](https://travis-ci.org/anowell/arest.svg)](https://travis-ci.org/anowell/arest)

## Build

    cargo build
    docker build -t arest .

# Running it

   $ docker run -p 8080:8080 arest
   $ curl -s localhost:8080/data -XPOST -d"
       An old silent pond...
       A frog jumps into the pond,
       splash! Silence again.
       "
   Words counted:       4      13     102

