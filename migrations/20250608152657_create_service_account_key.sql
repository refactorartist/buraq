-- Add migration script here
CREATE TABLE service_account_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_account_id UUID NOT NULL,
    algorithm VARCHAR(10) NOT NULL,
    key VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE INDEX idx_service_account_keys_service_account_id ON service_account_keys(service_account_id);
CREATE INDEX idx_service_account_keys_key ON service_account_keys(key);
CREATE INDEX idx_service_account_keys_expires_at ON service_account_keys(expires_at);
