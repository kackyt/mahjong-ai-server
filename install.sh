#!/bin/bash

set -e

groupadd -r app && useradd -r -g app app

# echo deb http://ftp.us.debian.org/debian/ bookworm main contrib non-free >> /etc/apt/sources.list
# echo deb http://ftp.us.debian.org/debian/ bookworm-proposed-updates main contrib  non-free >> /etc/apt/sources.list
# echo deb http://ftp.us.debian.org/debian/ bookworm-updates main contrib >> /etc/apt/sources.list
# echo deb http://ftp.us.debian.org/debian/ bookworm-backports main contrib  non-free >> /etc/apt/sources.list
apt-get update -y
apt-get install -y --no-install-recommends curl \
    ca-certificates \
    libssl-dev \
    python3-crcmod \
    apt-transport-https \
    lsb-release \
    openssh-client \
    gnupg

# Cloud SDKインストール
echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] http://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list
curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key --keyring /usr/share/keyrings/cloud.google.gpg  add -
apt-get update -y
apt-get install google-cloud-sdk -y --no-install-recommends

rm -rf /var/lib/apt/lists/*
apt-get clean

mkdir -p ${APP_PATH}
chown app:app ${APP_PATH} -R
