-- Add migration script here
CREATE TABLE project_accesses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    environment_id UUID NOT NULL,
    service_account_id UUID,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

-- Create a junction table for the many-to-many relationship between project_accesses and project_scopes
CREATE TABLE project_access_scopes (
    project_access_id UUID NOT NULL,
    project_scope_id UUID NOT NULL,
    PRIMARY KEY (project_access_id, project_scope_id)
);

CREATE INDEX idx_project_accesses_environment_id ON project_accesses(environment_id);
CREATE INDEX idx_project_accesses_service_account_id ON project_accesses(service_account_id);
CREATE INDEX idx_project_access_scopes_project_access_id ON project_access_scopes(project_access_id);
CREATE INDEX idx_project_access_scopes_project_scope_id ON project_access_scopes(project_scope_id);
