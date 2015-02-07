FROM ubuntu:14.04

COPY target/arest /bin/arest

EXPOSE 8080
CMD ["/bin/arest"]