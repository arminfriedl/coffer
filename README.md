# Coffer 
[![Build Status](https://drone.friedl.net/api/badges/incubator/coffer/status.svg?ref=refs/heads/develop)](https://drone.friedl.net/incubator/coffer)

Coffer is a collection of tools for the simple but secure management of
application configuration.

It is meant to be flexible and simple, hence does not assume much about your
environment. Especially, you don't need a kubernetes cluster for running coffer.
Coffer runs directly on your server, just as well as in a containerized setup or
a full kubernetes cluster. The only thing Coffer needs is a TCP connection
between the `coffer-server` (securely holding your configuration) and the
`coffer-client` (retrieving configuration and setting up your application).

## Overview
![](overview.png)

## The Parts of Coffer
Coffer is split into 3 binaries and a supporting library:

* `coffer-server`: The `coffer-server` securely stores configuration data and
  hands them out to `coffer-clients` upon request
* `coffer-client`: A `coffer-client` requests configuration data from a
  `coffer-server`, configures the application and may also start it up
* `coffer-companion`: A helper for generating certificates and encrypting
  configuration data
* `coffer-common`: A common library for all binaries containing common
  cryptographic operations, protocol code and interface definitions

## Security
Coffer does not rely on a secure connection or any specifics of the environment.
Instead security is provided by a basic public key cryptography scheme. This
gives you, the user, the flexibility to set up your ecosystem according to your
own security needs.

### Certificates
Certificates in coffer are the just a keypair consisting of a public and private
key as used by public key cryptography. So, basically, a certificate is nothing
more than a tuple of two primes.

Certificates in coffer can be generated by the `coffer-companion`. Every
`coffer-server` and `coffer-client` need their own certificate. Security
in coffer squarely depends on these certificates being kept secret.

### Configuration
Configuration can be written in [toml](https://github.com/toml-lang/toml)
format. It is secured by encrypting it with the public key of the
`coffer-server`. This can be done by invoking the `coffer-companion`.

Encrypted configuration can be conveniently stored in VCS (e.g. via git-lfs)
with your application. As long as the server certificate stays private.

### Communication
The communication between a `coffer-server` and a `coffer-client` is not and
does not need to be secured. A `coffer-server` associates configuration data
with the public keys of `coffer-clients`. Upon request of configuration data the
server sends back the configuration encrypted with the clients public key. Only
a client in posession of the corresponding private key can read the response.

### Trust Anchors
It is worth mentioning some things about trust anchors. Every cryptography
scheme, no matter how sophisticated, needs, at some point, something that can be
trusted. In case of HTTPS this is provided by central certificate authorities.
In case of the encrypted letters to your best friend this might be a pre-shared
password transmitted over phone in 1972.

From a coffer perspective keys can be trusted. That is, coffer assumes that
certificates are distributed and kept secret according to your threat model. An
attacker in control of a certificate can steal secret configuration!

Coffer does not assume a trust anchor for you. Instead, you are free to choose
your own trust anchor. In a simple personal server setup this might
mean just distributing certificates by hand. In a more complex, corporate
environment you may want to set up a secure, central authority. 

Trust anchors are a trade-off between convenience, complexity and security.
Coffer lets _you_ choose where along these axis you put your trust anchor.

