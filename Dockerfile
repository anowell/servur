FROM ubuntu:14.04

RUN groupadd -r arunner && useradd -m -g arunner arunner
COPY target/arest /bin/arest

EXPOSE 8080
ENTRYPOINT ["/bin/arest"]
# TODO: EntryPoint needs to become a script that supports:
# 1) Pre-init (e.g. /etc/preinit.d/*) for dependency loading
# 2) Starting the arest server
