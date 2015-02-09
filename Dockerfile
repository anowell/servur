FROM ubuntu:14.04

RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -r arunner && useradd -m -g arunner arunner
ADD ship /bin/ship

ENV AREST_VERSION 0.1.2
RUN curl -Lo /bin/arest.gz https://github.com/anowell/arest/releases/download/v$AREST_VERSION/arest.gz && \
    gunzip /bin/arest.gz && \
    chmod 755 /bin/arest

EXPOSE 8080
ENTRYPOINT ["/bin/ship"]
