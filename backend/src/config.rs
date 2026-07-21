#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://letsorder.db?mode=rwc".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8080);
        let allowed_origins = std::env::var("LETSORDER_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://127.0.0.1:5173,http://localhost:5173".to_string())
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        if std::env::var("LETSORDER_ENV").as_deref() == Ok("production") {
            let password = std::env::var("LETSORDER_ADMIN_PASSWORD").map_err(|_| {
                anyhow::anyhow!("LETSORDER_ADMIN_PASSWORD is required in production")
            })?;
            validate_production_admin_password(&password)?;
        }

        Ok(Self {
            database_url,
            port,
            allowed_origins,
        })
    }
}

fn validate_production_admin_password(password: &str) -> anyhow::Result<()> {
    if password == "Admin_1234" || password.len() < 12 {
        return Err(anyhow::anyhow!(
            "LETSORDER_ADMIN_PASSWORD must be changed and at least 12 characters"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_production_admin_password;

    #[test]
    fn production_rejects_default_and_short_admin_passwords() {
        assert!(validate_production_admin_password("Admin_1234").is_err());
        assert!(validate_production_admin_password("short-password").is_ok());
        assert!(validate_production_admin_password("short").is_err());
    }
}
