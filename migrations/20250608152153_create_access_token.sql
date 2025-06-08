-- Add migration script here
CREATE TABLE access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_access_id UUID NOT NULL,
    key VARCHAR(255) NOT NULL,
    algorithm VARCHAR(10) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_access_tokens_project_access_id ON access_tokens(project_access_id);
CREATE INDEX idx_access_tokens_key ON access_tokens(key);
CREATE INDEX idx_access_tokens_expires_at ON access_tokens(expires_at);
