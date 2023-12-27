FROM balenalib/i386-ubuntu:bionic-build

ENV APP_PATH /opt/apps
ENV HOME /root

COPY . ${APP_PATH}
WORKDIR ${APP_PATH}

RUN cd loadlibrary && make && cp mpclient ${APP_PATH}
ENTRYPOINT [ "/bin/sh", "-c" ]
CMD ["./mpclient", "test.dll"]
