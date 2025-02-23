## Matrix

[Matrix](https://matrix.org/) is a rich ecosystem of clients, servers, bots and
application services. It defines a set of open APIs for decentralized
communication, suitable for securely publishing, persisting and subscribing to
data over a global open federation of servers with no single point of control.
Uses include Instant Messaging (IM), Voice over IP (VoIP) signalling, Internet
of Things (IoT) communication, and bridging together existing communication
silos—providing the basis of a new, open, real-time communication ecosystem.

# Spelt

Spelt aims to be a server implementation of the Matrix API. The following are
the relevant components of the specification:

* [Matrix client-server
  specification](https://spec.matrix.org/v1.13/client-server-api/): provides
  messaging functionality used by Matrix-compliant clients (target version
  1.13)

* [Matrix server-server
  specification](https://spec.matrix.org/v1.13/server-server-api/):
  provides federation amongst servers (target version 1.13)

Spelt is implemented in [Rust](https://www.rust-lang.org/) using
[Actix](https://actix.rs/) as the web app framework and
[PostgreSQL](https://postgresql.org/) as the database.

## Setup

0. If you haven't previously started the Surreal database server, start it now
   with your persistent storage engine of choice. Here is an example using the
   RocksDB engine. (Subsequent starts can omit the user and password as they will
   have been written to disk.)
    - `surreal start --user root --pass 'my-secret' rocksdb:///home/$LOGUSER/lib/surrealdb/surreal.db`
1. Start the Surreal CLI as root and create the namespace user for the Spelt
   app. Modify the following as needed.
    - `surreal sql --user root --pass 'my-secret' --ns spelt --db test --pretty --endpoint http://localhost:8000/`
    - `USE NS spelt;`
    - `DEFINE USER spelt_app ON NAMESPACE PASSWORD 'my-secret' ROLES EDITOR;`
2. Copy `config/app.example.toml` to `config/app.toml` and update values as
   appropriate.
3. In `config/`, run the following to generate a pair of private and public key
   files for signing JWTs:
    - `openssl genpkey -outform pem -algorithm rsa -out pkey.pem`
    - `openssl rsa -in pkey.pem -pubout -out public.pem`

## License

Spelt is licensed under the three-clause BSD license. See LICENSE.txt.

## To Do

API implementation progress is tracked in [TODO.md](TODO.md).
