use anyhow::Result;
use sqlx::{Connection, FromRow, Sqlite, SqliteConnection};

#[tokio::test]
async fn dump_db() -> Result<()> {
    let mut con = SqliteConnection::connect("./zscm.db").await?;
    let sql = "select nickname, usercode from sh_user where deleted = '0'";
    let rows = sqlx::query_as::<Sqlite, (String, String)>(sql)
        .fetch_all(&mut con)
        .await?;
    let i_sql = "insert into reporter(name, ref_code) values(?,?)";
    for r in rows.into_iter() {
        sqlx::query(i_sql)
            .bind(r.0)
            .bind(r.1)
            .execute(&mut con)
            .await?;
    }

    assert!(true);
    Ok(())
}
