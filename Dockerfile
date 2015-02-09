FROM ubuntu:14.04

RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -r arunner && useradd -m -g arunner arunner

ENV AREST_VERSION 0.1.1
RUN curl -o /bin/arest.gz https://github.com/anowell/arest/releases/download/v$AREST_VERSION/arest.gz && \
    gunzip /bin/arest.gz && \
    chmod 755 /bin/arest

EXPOSE 8080
ENTRYPOINT ["/bin/arest"]
# TODO: EntryPoint needs to become a script that supports:
# 1) Pre-init (e.g. /etc/preinit.d/*) for dependency loading
# 2) Starting the arest server
