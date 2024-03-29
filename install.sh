#!/bin/bash

set -e

groupadd -r -g 1000 app && useradd -r -u 1000 -m -s /bin/bash -g app app
mkdir -p /usr/share/man/man1

mkdir -p /home/app/.config/gcloud/configurations
chown -R app:app /home/app
chmod -R 700 /home/app/.config

mkdir ${APP_PATH}
chown -R app:app ${APP_PATH}
chmod -R 700 ${APP_PATH}

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
