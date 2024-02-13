FROM kackyt/rust-cmake-devel:0.4.0 as builder

ENV DEBIAN_FRONTEND noninteractive
ENV HOME /home/app

USER app

COPY . ${HOME}
WORKDIR ${HOME}
RUN cargo build -p server --release --features load-dll

FROM debian:bookworm-slim
ENV APP_PATH /opt/apps

COPY ./install.sh .
RUN set -e && bash ./install.sh

USER app

COPY --from=builder /home/app/target/release/server ${APP_PATH}/server
WORKDIR ${APP_PATH}

ENTRYPOINT [ "./server" ]
