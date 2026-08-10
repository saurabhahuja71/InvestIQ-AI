-- Email OTP verification for account registration.
-- Mirrors the password-reset pattern: no SMTP yet, so the generated code is
-- returned to the app in the API response and hashed at rest here.

CREATE TABLE IF NOT EXISTS email_verification_otps (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email      TEXT NOT NULL,
    otp_hash   TEXT NOT NULL,
    attempts   INT NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at    TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_verification_otps_email ON email_verification_otps (email);
CREATE INDEX IF NOT EXISTS idx_email_verification_otps_expires ON email_verification_otps (expires_at);
