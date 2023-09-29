pub struct DbClient {
    pub pool: sqlx::PgPool,
}

impl DbClient {
    pub async fn new(dsn: &str) -> sqlx::Result<Self> {
        let pool = sqlx::PgPool::connect(dsn).await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!().run(&self.pool).await
    }
}
