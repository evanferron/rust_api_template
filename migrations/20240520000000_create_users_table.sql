-- Create extension for UUID
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Créer la table user sans le champ is_active
CREATE TABLE IF NOT EXISTS "user" (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Créer un index sur le champ email
CREATE INDEX IF NOT EXISTS idx_user_email ON "user"(email);