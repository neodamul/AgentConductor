use super::Store;

impl Store {
    /// Next 128 pending observations; remaining rows stay durable for a later sync.
    pub fn pending_memory(&self) -> Result<Vec<(i64, String)>, String> {
        let mut query = self
            .0
            .prepare("SELECT id,content FROM memory_outbox WHERE delivered=0 ORDER BY id LIMIT 128")
            .map_err(|e| e.to_string())?;
        query
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
    pub fn memory_delivered(&self, id: i64) -> Result<(), String> {
        let changed = self
            .0
            .execute("UPDATE memory_outbox SET delivered=1 WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err("unknown memory outbox item".into());
        }
        Ok(())
    }
    pub fn recall(&self, query: &str) -> Result<Vec<String>, String> {
        let query = query
            .split_whitespace()
            .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        if query.is_empty() {
            return Ok(vec![]);
        }
        let mut statement=self.0.prepare("SELECT content FROM memory_search WHERE memory_search MATCH ?1 ORDER BY rank LIMIT 20").map_err(|e|e.to_string())?;
        statement
            .query_map([query], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}
