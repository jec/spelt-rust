CREATE TABLE users (
    id                 BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name               VARCHAR(256)             NOT NULL UNIQUE,
    email              VARCHAR(1024)            NOT NULL UNIQUE,
    encrypted_password VARCHAR(1024)            NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE sessions (
    id                BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid              uuid                     NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    device_identifier VARCHAR(256)             NOT NULL UNIQUE,
    device_name       VARCHAR(256),
    user_id           BIGINT                   NOT NULL
        REFERENCES users (id),
    created_at        TIMESTAMP WITH TIME ZONE NOT NULL        DEFAULT NOW(),
    updated_at        TIMESTAMP WITH TIME ZONE NOT NULL        DEFAULT NOW()
);
