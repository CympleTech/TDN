use rusqlite::{params, types::Value, Connection};
use std::path::PathBuf;
use tdn_types::primitives::Result;

pub enum DsValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl DsValue {
    pub fn is_none(&self) -> bool {
        match self {
            DsValue::Null => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DsValue::Text(s) => &s,
            _ => "",
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            DsValue::Integer(i) => i == &1i64,
            _ => false,
        }
    }

    pub fn as_i64(self) -> i64 {
        match self {
            DsValue::Integer(i) => i,
            _ => 0,
        }
    }

    pub fn as_string(self) -> String {
        match self {
            DsValue::Text(s) => s,
            _ => "".to_owned(),
        }
    }

    pub fn as_f64(self) -> f64 {
        match self {
            DsValue::Real(f) => f,
            _ => 0.0,
        }
    }

    pub fn as_vec(self) -> Vec<u8> {
        match self {
            DsValue::Blob(v) => v,
            _ => vec![],
        }
    }
}

impl From<Value> for DsValue {
    fn from(value: Value) -> DsValue {
        match value {
            Value::Null => DsValue::Null,
            Value::Integer(i) => DsValue::Integer(i),
            Value::Real(i) => DsValue::Real(i),
            Value::Text(s) => DsValue::Text(s),
            Value::Blob(v) => DsValue::Blob(v),
        }
    }
}

pub struct DStorage {
    connect: Connection,
}

impl DStorage {
    #[inline]
    pub fn open(path: PathBuf, passwd: &str) -> Result<Self> {
        let connect = Connection::open(path)?;
        connect.pragma_update(None, "key", passwd)?;
        Ok(DStorage { connect })
    }

    /// tmp use.
    #[inline]
    pub fn execute(&self, sql: &str) -> Result<usize> {
        Ok(self.connect.execute(sql, params![])?)
    }

    /// tmp use.
    #[inline]
    pub fn query(&self, sql: &str) -> Result<Vec<Vec<DsValue>>> {
        let mut stmt = self.connect.prepare(sql)?;
        let mut rows = stmt.query(params![])?;

        let mut matrix: Vec<Vec<DsValue>> = Vec::new();
        while let Some(row) = rows.next()? {
            let mut values: Vec<DsValue> = Vec::new();
            for i in 0..row.as_ref().column_count() {
                values.push(row.get::<usize, Value>(i)?.into());
            }
            matrix.push(values);
        }

        Ok(matrix)
    }

    /// tmp use. return the insert id.
    #[inline]
    pub fn insert(&self, sql: &str) -> Result<i64> {
        self.execute(sql)?;
        Ok(self.connect.last_insert_rowid())
    }

    /// tmp use.
    #[inline]
    pub fn update(&self, sql: &str) -> Result<usize> {
        self.execute(sql)
    }

    /// tmp use.
    #[inline]
    pub fn delete(&self, sql: &str) -> Result<usize> {
        self.execute(sql)
    }

    /// tmp use.
    #[inline]
    pub fn flush(&self) -> Result<()> {
        self.connect.flush_prepared_statement_cache();
        Ok(())
    }

    /// tmp use.
    #[inline]
    pub fn close(self) -> Result<()> {
        self.flush()?;
        let _ = self.connect.close();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn auto_locol() {
        let db = DStorage::open(PathBuf::from("./test.db"), "test123").unwrap();
        db.execute("CREATE TABLE users(id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, name TEXT NOT NULL, age INTEGER NOT NULL);").unwrap();
        let id = db
            .insert("INSERT INTO users (name, age) values ('sun', 18)")
            .unwrap();
        db.close().unwrap();

        let db = DStorage::open(PathBuf::from("./test.db"), "test123").unwrap();
        let mut matrix = db
            .query(&format!("SELECT * FROM users WHERE id = {}", id))
            .unwrap();

        assert_eq!(matrix.len(), 1);
        let mut values = matrix.pop().unwrap();
        assert_eq!(values.pop().unwrap().as_i64(), 18);
        assert_eq!(values.pop().unwrap().as_str(), "sun");

        let lines = db
            .update(&format!("UPDATE users SET age = {} WHERE id = {}", 20, id))
            .unwrap();
        assert_eq!(lines, 1);

        let mut matrix = db
            .query(&format!("SELECT age FROM users WHERE id = {}", id))
            .unwrap();

        assert_eq!(matrix.len(), 1);
        let mut values = matrix.pop().unwrap();
        assert_eq!(values.pop().unwrap().as_i64(), 20);

        db.delete(&format!("DELETE FROM users WHERE id = {}", id))
            .unwrap();

        let matrix = db
            .query(&format!("SELECT * FROM users WHERE id = {}", id))
            .unwrap();
        assert_eq!(matrix.len(), 0);

        std::fs::remove_file("./test.db").unwrap();
    }
}
