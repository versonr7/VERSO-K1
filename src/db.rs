use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub language: String,
    pub last_opened: String,
}

pub struct ProjectDB {
    conn: Connection,
}

impl ProjectDB {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("/data/data/rust.verso_k1/databases/projects.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                language TEXT,
                last_opened TEXT
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn add_project(&self, name: &str, path: &str, language: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO projects (name, path, language, last_opened) 
             VALUES (?1, ?2, ?3, datetime('now'))",
            [name, path, language],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, language, last_opened 
             FROM projects ORDER BY last_opened DESC",
        )?;
        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    language: row.get(3)?,
                    last_opened: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(projects)
    }

    pub fn remember_project(&self, name: &str, path: &str, language: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO projects (name, path, language, last_opened) 
             VALUES (?1, ?2, ?3, datetime('now'))",
            [name, path, language],
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_create_and_insert() {
        let db = ProjectDB::new().unwrap();
        let id = db.add_project("TestProj", "/test", "rust").unwrap();
        assert!(id > 0);
        let projects = db.get_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "TestProj");
    }

    #[test]
    fn test_remember_project() {
        let db = ProjectDB::new().unwrap();
        db.remember_project("MyApp", "/data/myapp", "python")
            .unwrap();
        let projects = db.get_projects().unwrap();
        assert_eq!(projects[0].language, "python");
    }
}
