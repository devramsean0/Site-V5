use log::debug;
use rusqlite::Connection;

struct Migration {
    name: &'static str,
    sql: &'static str,
}
fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            name: "create_web_cache",
            sql: "CREATE TABLE IF NOT EXISTS web_cache (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                path TEXT NOT NULL,
                body TEXT NOT NULL,
                UNIQUE(path)
            )",
        }
    ]
}
pub fn run_migrations(con: &rusqlite::Connection) {
    let _ = con.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            hash TEXT NOT NULL,
            UNIQUE(id, hash)
        )",
        [],
    );
    for migration in migrations() {
        let hash = format!("{:x}", md5::compute(migration.sql));
        let str_hash = hash.as_str();
        let mut stmt = con.prepare("SELECT COUNT(*) FROM migrations WHERE name = ? OR hash = ?").unwrap();
        let count: i64 = stmt.query_row(&[&migration.name, str_hash], |row| row.get(0)).unwrap();
        if count == 0 {
            con.execute(migration.sql, []).unwrap();
            con.execute("INSERT INTO migrations (name, hash) VALUES (?, ?)", &[&migration.name, str_hash]).unwrap();
            debug!("Applied migration: {}", migration.name);
        }
    }
}


pub fn retrieve_web_cache(db: &Connection, method: String, path: String) -> Vec<WebCache> {
    let cache_lookup_sql = format!("SELECT * FROM web_cache WHERE method == \"{}\" AND path == \"{}\"", method, path);
    let cache_lookup = db
        .prepare(cache_lookup_sql.as_str()).unwrap()
        .query_map([], |row| {
            Ok(WebCache {
                id: row.get(0)?,
                method: row.get(1)?,
                path: row.get(2)?,
                body: row.get(3)?,
            })
        }).unwrap()
        .map(|x| x.unwrap())
        .collect::<Vec<WebCache>>();
    debug!("Retrieved Cache from DB");
    cache_lookup
}

pub fn insert_web_cache(db: &Connection, method: String, path: String, body: String) {
    let insert_sql = format!("INSERT INTO web_cache (method, path, body) VALUES (\"{}\", \"{}\", \"{}\")", method, path, body);
    db.execute(insert_sql.as_str(), []).unwrap();
    debug!("Inserted Cache into DB");
}

pub struct WebCache {
    id: i32,
    method: String,
    path: String,
    pub body: String,
}