FROM kackyt/rust-cmake-devel:0.3.0

ENV APP_PATH /opt/apps
ENV DEBIAN_FRONTEND noninteractive
ENV HOME /home/app

USER root

COPY . ${APP_PATH}
WORKDIR ${APP_PATH}

RUN apt-get install -y pkg-config

RUN chown app:app ${APP_PATH} -R

USER app
WORKDIR ${APP_PATH}
RUN cargo build -p server

ENTRYPOINT [ "/bin/sh", "-c" ]
CMD ["cargo run -p server ${APP_PATH}/Test.dll"]
