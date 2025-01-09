FROM docker.io/ubuntu:24.04
ARG TARGETARCH

RUN mkdir -p /opt/eb-event-logger \
 && apt-get -y update \
 && apt-get -y install ca-certificates \ 
 && apt-get -y clean \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /opt/eb-event-logger
COPY ./tools/target_arch.sh /opt/eb-event-logger
COPY ./tools/entrypoint.sh /
RUN --mount=type=bind,target=/context \
 cp /context/target/$(/opt/eb-event-logger/target_arch.sh)/release/eb-event-logger /opt/eb-event-logger/
CMD ["/entrypoint.sh"]
EXPOSE 8443
