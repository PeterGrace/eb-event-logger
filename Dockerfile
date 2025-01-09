FROM docker.io/ubuntu:24.04
ARG TARGETARCH

RUN mkdir -p /opt/eb-event-logger
WORKDIR /opt/eb-event-logger
COPY ./tools/target_arch.sh /opt/eb-event-logger
RUN --mount=type=bind,target=/context \
 cp /context/target/$(/opt/eb-event-logger/target_arch.sh)/release/eb-event-logger /opt/eb-event-logger/
CMD ["/opt/eb-event-logger/eb-event-logger"]
EXPOSE 8443
