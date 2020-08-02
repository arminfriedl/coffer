FROM gcr.io/distroless/static

COPY ./coffer-server .
COPY ./server.cert .
COPY ./config.enc .


EXPOSE 9187
ENTRYPOINT ["./coffer-server"]

CMD ["--certificate", "server.cert", \
     "--secrets", "config.enc", \
     "--address", "0.0.0.0:9187"]
