-- Add migration script here
CREATE TABLE server_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(255) NOT NULL,
    environment_id UUID NOT NULL,
    algorithm VARCHAR(10) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX idx_server_keys_environment_id ON server_keys(environment_id);
CREATE INDEX idx_server_keys_key ON server_keys(key);
