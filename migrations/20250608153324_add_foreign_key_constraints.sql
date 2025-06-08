-- Add migration script here

-- Foreign keys for environments table
ALTER TABLE environments
    ADD CONSTRAINT fk_environments_project_id
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE;

-- Foreign keys for project_scopes table
ALTER TABLE project_scopes
    ADD CONSTRAINT fk_project_scopes_project_id
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE;

-- Foreign keys for project_accesses table
ALTER TABLE project_accesses
    ADD CONSTRAINT fk_project_accesses_environment_id
    FOREIGN KEY (environment_id) REFERENCES environments(id) ON DELETE CASCADE;

ALTER TABLE project_accesses
    ADD CONSTRAINT fk_project_accesses_service_account_id
    FOREIGN KEY (service_account_id) REFERENCES service_accounts(id) ON DELETE SET NULL;

-- Foreign keys for project_access_scopes junction table
ALTER TABLE project_access_scopes
    ADD CONSTRAINT fk_project_access_scopes_project_access_id
    FOREIGN KEY (project_access_id) REFERENCES project_accesses(id) ON DELETE CASCADE;

ALTER TABLE project_access_scopes
    ADD CONSTRAINT fk_project_access_scopes_project_scope_id
    FOREIGN KEY (project_scope_id) REFERENCES project_scopes(id) ON DELETE CASCADE;

-- Foreign keys for access_tokens table
ALTER TABLE access_tokens
    ADD CONSTRAINT fk_access_tokens_project_access_id
    FOREIGN KEY (project_access_id) REFERENCES project_accesses(id) ON DELETE CASCADE;

-- Foreign keys for server_keys table
ALTER TABLE server_keys
    ADD CONSTRAINT fk_server_keys_environment_id
    FOREIGN KEY (environment_id) REFERENCES environments(id) ON DELETE CASCADE;

-- Foreign keys for service_account_keys table
ALTER TABLE service_account_keys
    ADD CONSTRAINT fk_service_account_keys_service_account_id
    FOREIGN KEY (service_account_id) REFERENCES service_accounts(id) ON DELETE CASCADE;
