#!/bin/bash

gcloud storage cp ${AI_DOWNLOAD_URL} ./target.dll
gcloud storage cp ${PAIYAMA_DOWNLOAD_URL} ./paiyama --recursive

./server -i target.dll -l ./logs -p ./paiyama

gcloud storage cp ./logs ${LOGS_UPLOAD_URL} --recursive
