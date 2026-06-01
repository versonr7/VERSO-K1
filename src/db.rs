use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
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
        #[cfg(test)]
        let db_path = {
            let mut path = std::env::temp_dir();
            path.push("verso_k1_projects_test.db");
            path.to_str().unwrap().to_string()
        };
        
        #[cfg(not(test))]
        let db_path = "/data/data/rust.verso_k1/databases/projects.db";
        
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let conn = Connection::open(&db_path)?;
        
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", 10000)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                language TEXT,
                last_opened TEXT DEFAULT CURRENT_TIMESTAMP,
                code_snippets TEXT,
                ai_embeddings BLOB
            )",
            [],
        )?;
        
        Ok(Self { conn })
    }
    
    pub fn add_project(&self, name: &str, path: &str, language: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (name, path, language) VALUES (?1, ?2, ?3)",
            params![name, path, language],
        )?;
        Ok(())
    }
    
    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, language, last_opened FROM projects ORDER BY last_opened DESC"
        )?;
        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                language: row.get(3)?,
                last_opened: row.get(4)?,
            })
        })?;
        projects.collect()
    }
    
    pub fn remember_project(&self, name: &str, path: &str, language: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO projects (name, path, language) VALUES (?1, ?2, ?3)",
            params![name, path, language],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_db_create_and_insert() {
        let db = ProjectDB::new().unwrap();
        db.add_project("test_proj", "/tmp/test", "rust").unwrap();
        let projects = db.get_projects().unwrap();
        assert!(!projects.is_empty());
        assert!(projects.iter().any(|p| p.name == "test_proj"));
    }
    
    #[test]
    fn test_remember_project() {
        let db = ProjectDB::new().unwrap();
        db.remember_project("my_app", "/home/user/app", "python").unwrap();
        let projects = db.get_projects().unwrap();
        assert!(projects.iter().any(|p| p.name == "my_app"));
    }
}
