#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://letsorder.db?mode=rwc".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8080);

        if std::env::var("LETSORDER_ENV").as_deref() == Ok("production") {
            let password = std::env::var("LETSORDER_ADMIN_PASSWORD").map_err(|_| {
                anyhow::anyhow!("LETSORDER_ADMIN_PASSWORD is required in production")
            })?;
            if password == "Admin_1234" || password.len() < 12 {
                return Err(anyhow::anyhow!(
                    "LETSORDER_ADMIN_PASSWORD must be changed and at least 12 characters"
                ));
            }
        }

        Ok(Self { database_url, port })
    }
}
