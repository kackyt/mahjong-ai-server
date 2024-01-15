FROM kackyt/rust-cmake-devel

ENV APP_PATH /opt/apps
ENV DEBIAN_FRONTEND noninteractive
ENV HOME /home/app

USER root

RUN apt-get install strace

COPY . ${APP_PATH}
WORKDIR ${APP_PATH}

RUN chown app:app ${APP_PATH} -R

USER app
WORKDIR ${APP_PATH}
# RUN RUST_BACKTRACE=full cargo build

ENTRYPOINT [ "/bin/sh", "-c" ]
CMD ["cargo run -p server"]
