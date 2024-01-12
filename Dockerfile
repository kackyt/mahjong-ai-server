FROM i386/rust:slim-bullseye

ENV APP_PATH /opt/apps

RUN apt update && apt install -y cmake libc6-dev
RUN groupadd -r app && useradd -r -g app app
COPY . ${APP_PATH}
WORKDIR ${APP_PATH}

RUN chown app:app ${APP_PATH} -R

USER app
ENV HOME /home/app

RUN cd loadlibrary && cargo build

WORKDIR ${APP_PATH}/loadlibrary

ENTRYPOINT [ "/bin/sh", "-c" ]
CMD ["cargo test"]
