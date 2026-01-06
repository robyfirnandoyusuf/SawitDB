use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::{fs, path::{Path, PathBuf}};
use uuid::Uuid;

#[derive(Clone)]
pub struct Engine {
    root: PathBuf,
}

impl Engine {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root).with_context(|| format!("create root {}", root.display()))?;
        Ok(Self { root })
    }

    fn db_path(&self, db: &str) -> PathBuf {
        self.root.join(sanitize(db))
    }
    fn col_path(&self, db: &str, col: &str) -> PathBuf {
        self.db_path(db).join(format!("{}.json", sanitize(col)))
    }

    pub fn db_list(&self) -> Result<Vec<String>> {
        let mut out = vec![];
        for e in fs::read_dir(&self.root).context("read root")? {
            let e = e?;
            if e.file_type()?.is_dir() {
                if let Some(s) = e.file_name().to_str() {
                    out.push(s.to_string());
                }
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn db_create(&self, db: &str) -> Result<()> {
        let p = self.db_path(db);
        if p.exists() { return Err(anyhow!("db exists")); }
        fs::create_dir_all(&p)?;
        Ok(())
    }

    pub fn db_rename(&self, db: &str, new_db: &str) -> Result<()> {
        let oldp = self.db_path(db);
        let newp = self.db_path(new_db);
        if !oldp.exists() { return Err(anyhow!("db not found")); }
        if newp.exists() { return Err(anyhow!("target db exists")); }
        fs::rename(oldp, newp)?;
        Ok(())
    }

    pub fn db_drop(&self, db: &str) -> Result<()> {
        let p = self.db_path(db);
        if !p.exists() { return Err(anyhow!("db not found")); }
        fs::remove_dir_all(p)?;
        Ok(())
    }

    fn load_array_or_empty(&self, path: &Path) -> Result<Vec<Value>> {
        if !path.exists() { return Ok(vec![]); }
        let raw = fs::read_to_string(path)?;
        if raw.trim().is_empty() { return Ok(vec![]); }
        let v: Value = serde_json::from_str(&raw).context("invalid json")?;
        Ok(v.as_array().ok_or_else(|| anyhow!("collection must be array"))?.clone())
    }

    fn atomic_save(&self, path: &Path, v: &Value) -> Result<()> {
        let tmp = path.with_extension("json.tmp");
        let pretty = serde_json::to_string_pretty(v)?;
        fs::write(&tmp, pretty)?;
        fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn insert(&self, db: &str, col: &str, mut doc: Value) -> Result<String> {
        if !doc.is_object() { return Err(anyhow!("doc must be object")); }
        fs::create_dir_all(self.db_path(db))?;

        let id = match doc.get("id") {
            Some(Value::String(s)) => s.clone(),
            Some(_) => return Err(anyhow!("id must be string")),
            None => {
                let new_id = Uuid::new_v4().to_string();
                doc.as_object_mut().unwrap().insert("id".into(), Value::String(new_id.clone()));
                new_id
            }
        };

        let p = self.col_path(db, col);
        let mut docs = self.load_array_or_empty(&p)?;
        if docs.iter().any(|d| d.get("id").and_then(|x| x.as_str()) == Some(&id)) {
            return Err(anyhow!("duplicate id"));
        }
        docs.push(doc);
        self.atomic_save(&p, &Value::Array(docs))?;
        Ok(id)
    }

    pub fn get(&self, db: &str, col: &str, id: &str) -> Result<Option<Value>> {
        let p = self.col_path(db, col);
        let docs = self.load_array_or_empty(&p)?;
        Ok(docs.into_iter().find(|d| d.get("id").and_then(|x| x.as_str()) == Some(id)))
    }

    pub fn update_merge(&self, db: &str, col: &str, id: &str, patch: Value) -> Result<bool> {
        if !patch.is_object() { return Err(anyhow!("patch must be object")); }
        let patch_obj = patch.as_object().unwrap();

        let p = self.col_path(db, col);
        let mut docs = self.load_array_or_empty(&p)?;
        let mut found = false;

        for d in docs.iter_mut() {
            if d.get("id").and_then(|x| x.as_str()) == Some(id) {
                let obj = d.as_object_mut().ok_or_else(|| anyhow!("stored doc invalid"))?;
                for (k, v) in patch_obj.iter() {
                    if k == "id" { continue; }
                    obj.insert(k.clone(), v.clone());
                }
                found = true;
                break;
            }
        }
        if found {
            self.atomic_save(&p, &Value::Array(docs))?;
        }
        Ok(found)
    }

    pub fn delete(&self, db: &str, col: &str, id: &str) -> Result<bool> {
        let p = self.col_path(db, col);
        let mut docs = self.load_array_or_empty(&p)?;
        let before = docs.len();
        docs.retain(|d| d.get("id").and_then(|x| x.as_str()) != Some(id));
        let deleted = docs.len() != before;
        if deleted {
            self.atomic_save(&p, &Value::Array(docs))?;
        }
        Ok(deleted)
    }

    pub fn find_eq(&self, db: &str, col: &str, key: &str, value: &str) -> Result<Vec<Value>> {
        let p = self.col_path(db, col);
        let docs = self.load_array_or_empty(&p)?;
        Ok(docs.into_iter().filter(|doc| match doc.get(key) {
            Some(Value::String(s)) => s == value,
            Some(Value::Number(n)) => n.to_string() == value,
            Some(Value::Bool(b)) => b.to_string() == value,
            _ => false,
        }).collect())
    }
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c=='_' || c=='-' { c } else { '_' }).collect()
}
