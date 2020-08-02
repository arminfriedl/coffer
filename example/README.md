Here you can find a simple example on how to run coffer.

# Run the example

To run the example, simply execute:

```shell
docker-compose up
```

This should print out:

```shell
 @@@@@@@   @@@@@@   @@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@
@@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@@  @@@@@@@@
!@@       @@!  @@@  @@!       @@!       @@!       @@!  @@@
!@!       !@!  @!@  !@!       !@!       !@!       !@!  @!@
!@!       @!@  !@!  @!!!:!    @!!!:!    @!!!:!    @!@!!@!
!!!       !@!  !!!  !!!!!:    !!!!!:    !!!!!:    !!@!@!
:!!       !!:  !!!  !!:       !!:       !!:       !!: :!!
:!:       :!:  !:!  :!:       :!:       :!:       :!:  !:!
 ::: :::  ::::: ::   ::        ::        :: ::::  ::   :::
 :: :: :   : :  :    :         :        : :: ::    :   : :

PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
TERM=xterm
container=podman
HOSTNAME=f7a1614d8752
HOME=/root
CLIENT_SECRET=SECRETKEY
client
0
```

Where `CLIENT_SECRET` is the secret set into the environment of `printenv` in
the client. `printenv` is the coffer'ed process.

Note that if you connect to a shell in the client container you are not able to
retrieve the secret. The secret _only_ exists in the environment of the
coffer'ed process. It does not even exist in some parent process of the
coffer'ed process, as coffer reaps itself while starting the sub-process. This
is quite different from other alternatives that either set the secrets into the
environment of the container, or into a volume on the container. Both of which
are accessible (with more or less effort) from anyone that has access to the
container.

Also note how the whole server container, despite being able to service hundreds
of clients in parallel, uses less than 3 MB in total.

# Setup and Configuration
In this example we create a [coffer server](server/Dockerfile) with some [client secrets](config.toml) 
and a simple [client container](client/Dockerfile). The container are built, run and connected a shared 
network via [docker-compose](docker-compose.yml).

Both, the client and the server, need a certificate. The client for
authenticating with the server, and the server for decrypting the client
secrets. 

Certificates can be generated with:
```shell
coffer-companion certificate certificate.cert
```

Furthermore, the secrets need to be authorized. Even though a coffer server can
handle secrets for multiple clients, a client can only request its own secrets.
Secrets retrieval is authorized by the public key of the client. You can get the
public key by
```shell
coffer-companion info certificate.cert
```

The public key must be put into the `id` section of a secret. For
example, our [secrets](config.toml) contain a section like this:
```toml
[client]
id = "452B0788D966059B21DB04FF37BC6161072B15EA2CDF88A5040FEEAB89D1143A"
CLIENT_SECRET = "SECRETKEY"
```

Finally, the promise of coffer is that certificates are the fundamental trust
anchor. This also means they are the _only_ thing you have to care about for
security of your secrets. Consequently, [secrets themselves are encrypted](server/config.enc)
with the server certificate:
```shell
coffer-companion encrypt --certificate certificate.cert --out config.enc --yaml config.toml 
```

# Encoffering the client
You may have noticed that the actual client program (`printenv`) is not run
directly. Instead the [Dockerfile](client/Dockerfile) contains an entrypoint like this:
```Dockerfile
ENTRYPOINT ["coffer-client"]

CMD ["--certificate", "client.cert", \
     "--server-address", "server:9187", \
     "--", \
     "printenv"]
```

The coffer client will connect to the server and retrieve its secrets.
Afterwards it sets the secrets into the process environment and replaces its own
process image with the coffer'ed command. 

The major drawback of this approach is that you have to build your own images.
If you want to coffer your own software this is probably only a minor
inconvenience. For pre-build images like e.g. the official
[postgres](https://hub.docker.com/_/postgres) image this means you have to
derive your own image for and coffer the entrypoint. For postgres this might
look like:
```Dockerfile
FROM postgres:12-alpine

COPY ./coffer-client /usr/local/bin
COPY ./postgres.cert .

ENTRYPOINT ["coffer-client", \
            "--certificate", "postgres.cert", 
            "--server-address", "server:9187", 
            "--", 
            "docker-entrypoint.sh"]

CMD ["postgres"]

```
