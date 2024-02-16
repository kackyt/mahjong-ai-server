#!/bin/bash

if [ -z "$AI_DOWNLOAD_URL" ] || [ -z "$PAIYAMA_DOWNLOAD_URL" ] || [ -z "$LOGS_UPLOAD_URL" ]; then
  echo "必要な環境変数が設定されていません。"
  exit 1
fi

mkdir paiyama

gcloud storage cp ${AI_DOWNLOAD_URL} ./target.dll
gcloud storage cp ${PAIYAMA_DOWNLOAD_URL} ./paiyama --recursive

./server -i target.dll -l ./logs -p ./paiyama

gcloud storage cp ./logs ${LOGS_UPLOAD_URL} --recursive
