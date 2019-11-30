FROM gcr.io/distroless/static

COPY ./* /app/

EXPOSE 9187
ENTRYPOINT ["/app/coffer-server", "-k", \
                                  "-m", "/app/masterkey.cbor", \
                                  "-s", "/app/secrets.cbor", \
                                  "--ip", "0.0.0.0"]