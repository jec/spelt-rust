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

1. Create PostgreSQL development database and user. Below are examples. (Don't
   use SUPERUSER for the production database.)
    - `create database spelt_dev;`
    - `create role spelt_app superuser password 'my-secret';`
    - `grant all on database spelt_dev to spelt_app;`
2. Copy `config/app.example.toml` to `config/app.toml` and update values as
   appropriate.
3. In `config/`, run the following to generate a pair of private and public key
   files for signing JWTs:
    - `openssl genpkey -outform pem -algorithm rsa -out pkey.pem`
    - `openssl rsa -in pkey.pem -pubout -out public.pem`

## License

Spelt is licensed under the three-clause BSD license. See LICENSE.txt.

## To Do

Spelt is under active development, and much work remains before this becomes a
functioning messaging server.

### Client-Server

This checklist tracks the progress of implementing the endpoints defined in the
client-server spec.

- [ ] 3 Server discovery
    - [x] `GET /.well-known/matrix/client`
    - [x] `GET /_matrix/client/versions`
    - [ ] `GET /.well-known/matrix/support`
- [ ] 4 Client authentication
    - [x] `GET /_matrix/client/v1/register/m.login.registration_token/validity`
    - [x] `GET /_matrix/client/v3/login`
    - [ ] `POST /_matrix/client/v3/login`
    - [ ] `POST /_matrix/client/v1/login/get_token`
    - [ ] `POST /_matrix/client/v3/refresh`
    - [ ] `POST /_matrix/client/v3/logout`
    - [ ] `POST /_matrix/client/v3/logout/all`
    - [ ] `POST /_matrix/client/v3/account/deactivate`
    - [ ] `POST /_matrix/client/v3/account/password`
    - [ ] `POST /_matrix/client/v3/account/password/email/requestToken`
    - [ ] `POST /_matrix/client/v3/account/password/msisdn/requestToken`
    - [ ] `POST /_matrix/client/v3/register`
    - [ ] `GET /_matrix/client/v3/register/available`
    - [ ] `POST /_matrix/client/v3/register/email/requestToken`
    - [ ] `POST /_matrix/client/v3/register/msisdn/requestToken`
    - [ ] `GET /_matrix/client/v3/account/3pid`
    - [ ] `POST /_matrix/client/v3/account/3pid` _DEPRECATED_
    - [ ] `POST /_matrix/client/v3/account/3pid/add`
    - [ ] `POST /_matrix/client/v3/account/3pid/bind`
    - [ ] `POST /_matrix/client/v3/account/3pid/delete`
    - [ ] `POST /_matrix/client/v3/account/3pid/email/requestToken`
    - [ ] `POST /_matrix/client/v3/account/3pid/msisdn/requestToken`
    - [ ] `POST /_matrix/client/v3/account/3pid/unbind`
    - [ ] `GET /_matrix/client/v3/account/whoami`
- [ ] 5 Capabilities negotiation
    - [ ] `GET /_matrix/client/v3/capabilities`
- [ ] 6 Filtering
    - [ ] `POST /_matrix/client/v3/user/{userId}/filter`
    - [ ] `GET /_matrix/client/v3/user/{userId}/filter/{filterId}`
- [ ] 7 Events
    - [ ] `GET /_matrix/client/v3/sync`
    - [ ] `GET /_matrix/client/v3/events` _DEPRECATED_
    - [ ] `GET /_matrix/client/v3/events/{eventId}` _DEPRECATED_
    - [ ] `GET /_matrix/client/v3/initialSync` _DEPRECATED_
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/event/{eventId}`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/joined_members`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/members`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/state`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/state/{eventType}/{stateKey}`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/messages`
    - [ ] `GET /_matrix/client/v1/rooms/{roomId}/timestamp_to_event`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/initialSync` _DEPRECATED_
    - [ ] `PUT /_matrix/client/v3/rooms/{roomId}/state/{eventType}/{stateKey}`
    - [ ] `PUT /_matrix/client/v3/rooms/{roomId}/send/{eventType}/{txnId}`
    - [ ] `PUT /_matrix/client/v3/rooms/{roomId}/redact/{eventId}/{txnId}`
    - [ ] `GET /_matrix/client/v1/rooms/{roomId}/relations/{eventId}`
    - [ ] `GET /_matrix/client/v1/rooms/{roomId}/relations/{eventId}/{relType}`
    - [ ] `GET /_matrix/client/v1/rooms/{roomId}/relations/{eventId}/{relType}/{eventType}`
- [ ] 8 Rooms
    - [ ] `POST /_matrix/client/v3/createRoom`
    - [ ] `GET /_matrix/client/v3/directory/room/{roomAlias}`
    - [ ] `PUT /_matrix/client/v3/directory/room/{roomAlias}`
    - [ ] `DELETE /_matrix/client/v3/directory/room/{roomAlias}`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/aliases`
    - [ ] `GET /_matrix/client/v3/joined_rooms`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/invite`
    - [ ] `POST /_matrix/client/v3/join/{roomIdOrAlias}`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/join`
    - [ ] `POST /_matrix/client/v3/knock/{roomIdOrAlias}`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/forget`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/leave`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/kick`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/ban`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/unban`
    - [ ] `GET /_matrix/client/v3/directory/list/room/{roomId}`
    - [ ] `PUT /_matrix/client/v3/directory/list/room/{roomId}`
    - [ ] `GET /_matrix/client/v3/publicRooms`
    - [ ] `POST /_matrix/client/v3/publicRooms`
- [ ] 9 User Data
    - [ ] `POST /_matrix/client/v3/user_directory/search`
    - [ ] `GET /_matrix/client/v3/profile/{userId}`
    - [ ] `GET /_matrix/client/v3/profile/{userId}/avatar_url`
    - [ ] `PUT /_matrix/client/v3/profile/{userId}/avatar_url`
    - [ ] `GET /_matrix/client/v3/profile/{userId}/displayname`
    - [ ] `PUT /_matrix/client/v3/profile/{userId}/displayname`
- [ ] 10 Modules
    - [ ] `PUT /_matrix/client/v3/rooms/{roomId}/typing/{userId}`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/receipt/{receiptType}/{eventId}`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/read_markers`
    - [ ] `GET /_matrix/client/v3/presence/{userId}/status`
    - [ ] `PUT /_matrix/client/v3/presence/{userId}/status`
    - [ ] `GET /_matrix/client/v1/media/config`
    - [ ] `GET /_matrix/client/v1/media/download/{serverName}/{mediaId}`
    - [ ] `GET /_matrix/client/v1/media/download/{serverName}/{mediaId}/{fileName}`
    - [ ] `GET /_matrix/client/v1/media/preview_url`
    - [ ] `GET /_matrix/client/v1/media/thumbnail/{serverName}/{mediaId}`
    - [ ] `POST /_matrix/media/v1/create`

    - [ ] `GET /_matrix/media/v3/config` _DEPRECATED_
    - [ ] `GET /_matrix/media/v3/download/{serverName}/{mediaId}` _DEPRECATED_
    - [ ] `GET /_matrix/media/v3/download/{serverName}/{mediaId}/{fileName}` _DEPRECATED_
    - [ ] `GET /_matrix/media/v3/preview_url` _DEPRECATED_
    - [ ] `GET /_matrix/media/v3/thumbnail/{serverName}/{mediaId}` _DEPRECATED_
    - [ ] `POST /_matrix/media/v3/upload`
    - [ ] `PUT /_matrix/media/v3/upload/{serverName}/{mediaId}`
    - [ ] `PUT /_matrix/client/v3/sendToDevice/{eventType}/{txnId}`
    - [ ] `POST /_matrix/client/v3/delete_devices`
    - [ ] `GET /_matrix/client/v3/devices`
    - [ ] `GET /_matrix/client/v3/devices/{deviceId}`
    - [ ] `PUT /_matrix/client/v3/devices/{deviceId}`
    - [ ] `DELETE /_matrix/client/v3/devices/{deviceId}`
    - [ ] `POST /_matrix/client/v3/keys/device_signing/upload`
    - [ ] `POST /_matrix/client/v3/keys/signatures/upload`
    - [ ] `GET /_matrix/client/v3/room_keys/keys`
    - [ ] `PUT /_matrix/client/v3/room_keys/keys`
    - [ ] `DELETE /_matrix/client/v3/room_keys/keys`
    - [ ] `GET /_matrix/client/v3/room_keys/keys/{roomId}`
    - [ ] `PUT /_matrix/client/v3/room_keys/keys/{roomId}`
    - [ ] `DELETE /_matrix/client/v3/room_keys/keys/{roomId}`
    - [ ] `GET /_matrix/client/v3/room_keys/keys/{roomId}/{sessionId}`
    - [ ] `PUT /_matrix/client/v3/room_keys/keys/{roomId}/{sessionId}`
    - [ ] `DELETE /_matrix/client/v3/room_keys/keys/{roomId}/{sessionId}`
    - [ ] `GET /_matrix/client/v3/room_keys/version`
    - [ ] `POST /_matrix/client/v3/room_keys/version`
    - [ ] `GET /_matrix/client/v3/room_keys/version/{version}`
    - [ ] `PUT /_matrix/client/v3/room_keys/version/{version}`
    - [ ] `DELETE /_matrix/client/v3/room_keys/version/{version}`
    - [ ] `GET /_matrix/client/v3/keys/changes`
    - [ ] `POST /_matrix/client/v3/keys/claim`
    - [ ] `POST /_matrix/client/v3/keys/query`
    - [ ] `POST /_matrix/client/v3/keys/upload`
    - [ ] `GET /_matrix/client/v3/pushrules/`
    - [ ] `GET /_matrix/client/v3/pushrules/global/`
    - [ ] `GET /_matrix/client/v3/pushrules/global/{kind}/{ruleId}`
    - [ ] `PUT /_matrix/client/v3/pushrules/global/{kind}/{ruleId}`
    - [ ] `DELETE /_matrix/client/v3/pushrules/global/{kind}/{ruleId}`
    - [ ] `GET /_matrix/client/v3/pushrules/global/{kind}/{ruleId}/actions`
    - [ ] `PUT /_matrix/client/v3/pushrules/global/{kind}/{ruleId}/actions`
    - [ ] `GET /_matrix/client/v3/pushrules/global/{kind}/{ruleId}/enabled`
    - [ ] `PUT /_matrix/client/v3/pushrules/global/{kind}/{ruleId}/enabled`
    - [ ] `GET /_matrix/client/v3/pushers`
    - [ ] `POST /_matrix/client/v3/pushers/set`
    - [ ] `GET /_matrix/client/v3/notifications`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/invite`
    - [ ] `POST /_matrix/client/v3/search`
    - [ ] `GET /_matrix/client/v3/events`
    - [ ] `GET /_matrix/client/v3/user/{userId}/rooms/{roomId}/tags`
    - [ ] `PUT /_matrix/client/v3/user/{userId}/rooms/{roomId}/tags/{tag}`
    - [ ] `DELETE /_matrix/client/v3/user/{userId}/rooms/{roomId}/tags/{tag}`
    - [ ] `GET /_matrix/client/v3/user/{userId}/account_data/{type}`
    - [ ] `PUT /_matrix/client/v3/user/{userId}/account_data/{type}`
    - [ ] `GET /_matrix/client/v3/user/{userId}/rooms/{roomId}/account_data/{type}`
    - [ ] `PUT /_matrix/client/v3/user/{userId}/rooms/{roomId}/account_data/{type}`
    - [ ] `GET /_matrix/client/v3/rooms/{roomId}/context/{eventId}`
    - [ ] `GET /_matrix/client/v3/login/sso/redirect`
    - [ ] `GET /_matrix/client/v3/login/sso/redirect/{idpId}`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/report`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/report/{eventId}`
    - [ ] `GET /_matrix/client/v3/thirdparty/location`
    - [ ] `GET /_matrix/client/v3/thirdparty/location/{protocol}`
    - [ ] `GET /_matrix/client/v3/thirdparty/protocol/{protocol}`
    - [ ] `GET /_matrix/client/v3/thirdparty/protocols`
    - [ ] `GET /_matrix/client/v3/thirdparty/user`
    - [ ] `GET /_matrix/client/v3/thirdparty/user/{protocol}`
    - [ ] `POST /_matrix/client/v3/user/{userId}/openid/request_token`
    - [ ] `POST /_matrix/client/v3/rooms/{roomId}/upgrade`
    - [ ] `GET /_matrix/client/v1/rooms/{roomId}/hierarchy`
    - [ ] `GET /_matrix/client/v1/rooms/{roomId}/threads`

### Server-Server

The relevant endpoints for implementing the federation specification will
follow eventually.
