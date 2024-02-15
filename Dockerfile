FROM kackyt/rust-cmake-devel:0.4.0 as builder

ENV DEBIAN_FRONTEND noninteractive
ENV HOME /home/app

USER app

COPY . ${HOME}
WORKDIR ${HOME}
RUN cargo build -p server --release --features load-dll

FROM debian:bookworm-slim as runtime
ENV APP_PATH /opt/apps

COPY ./install.sh .
RUN set -e && bash ./install.sh

USER app
WORKDIR ${APP_PATH}

COPY --from=builder /home/app/target/release/server ${APP_PATH}/server
COPY ./run.sh ${APP_PATH}

ENTRYPOINT [ "bash", "-c" ]
CMD [ "bash ./run.sh" ]
