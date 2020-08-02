FROM busybox

RUN mkdir -p /usr/local/bin

COPY ./coffer-client /usr/local/bin
COPY ./client.cert .

ENTRYPOINT ["coffer-client"]

CMD ["--certificate", "client.cert", \
     "--server-address", "server:9187", \
     "--", \
     "printenv"]
