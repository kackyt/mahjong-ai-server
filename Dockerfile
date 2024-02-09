FROM kackyt/rust-cmake-devel:0.3.0

ENV APP_PATH /opt/apps
ENV DEBIAN_FRONTEND noninteractive
ENV HOME /home/app

USER root

COPY . ${APP_PATH}
WORKDIR ${APP_PATH}

RUN apt-get install -y pkg-config
RUN apt-get install -y \
    curl \

    python3-crcmod \
    apt-transport-https \
    lsb-release \
    openssh-client \ 
    gnupg

# Cloud SDKインストール
RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] http://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && \
    curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key --keyring /usr/share/keyrings/cloud.google.gpg  add - && \
    apt-get update -y && \
    apt-get install google-cloud-sdk -y

RUN chown app:app ${APP_PATH} -R

USER app
WORKDIR ${APP_PATH}
RUN cargo build -p server

ENTRYPOINT [ "/bin/sh", "-c" ]
CMD ["cargo run -p server ${APP_PATH}/Test.dll"]
