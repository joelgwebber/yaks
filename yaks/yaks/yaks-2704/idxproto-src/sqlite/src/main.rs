//! SQLite FTS5 prototype: FTS5 table (contentless, prefix indexes), edges + docs tables,
//! optional identifier-aware tokenizer (env IDTOK=1) wrapped around unicode61.
use common::harness::{self, Backend};
use common::Doc;
use std::collections::HashMap;
use std::path::Path;
use rusqlite::{params, Connection};
use std::time::{Duration, Instant};

mod idtok;

pub struct Sq {
    c: Connection,
}

fn tokenizer() -> String {
    if std::env::var("IDTOK").is_ok() {
        "idtok unicode61 remove_diacritics 2".to_string()
    } else {
        "unicode61 remove_diacritics 2".to_string()
    }
}

fn stored() -> bool {
    std::env::var("STORED").is_ok()
}

fn connect(dir: &Path) -> Connection {
    let c = Connection::open(dir.join("index.db")).unwrap();
    idtok::register(&c);
    c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;").unwrap();
    c
}

impl Backend for Sq {
    const NAME: &'static str = "sqlite-fts5";
    fn create(dir: &Path) -> Self {
        let c = connect(dir);
        let content = if stored() { "" } else { ", content='', contentless_delete=1" };
        c.execute_batch(&format!(
            "CREATE TABLE docs(id INTEGER PRIMARY KEY, path TEXT NOT NULL UNIQUE, kind TEXT NOT NULL, title TEXT NOT NULL, mtime INTEGER NOT NULL);
             CREATE TABLE edges(src INTEGER NOT NULL, target TEXT NOT NULL, anchor TEXT NOT NULL);
             CREATE VIRTUAL TABLE fts USING fts5(kind, title, tags, body, tokenize='{}'{}{});",
            tokenizer(),
            if std::env::var("NOPREFIX").is_ok() { "" } else { ", prefix='2 3 4'" },
            content
        ))
        .unwrap();
        Sq { c }
    }
    fn open(dir: &Path) -> Self {
        Sq { c: connect(dir) }
    }
    fn build(&mut self, docs: &[Doc], times: &HashMap<String, i64>) -> Vec<(&'static str, Duration)> {
        let t0 = Instant::now();
        let tx = self.c.transaction().unwrap();
        {
            let mut d = tx.prepare("INSERT INTO docs(path,kind,title,mtime) VALUES(?1,?2,?3,?4)").unwrap();
            let mut f = tx.prepare("INSERT INTO fts(rowid,kind,title,tags,body) VALUES(?1,?2,?3,?4,?5)").unwrap();
            let mut e = tx.prepare("INSERT INTO edges(src,target,anchor) VALUES(?1,?2,?3)").unwrap();
            for doc in docs {
                d.execute(params![doc.path, doc.kind, doc.title, times[&doc.path]]).unwrap();
                let id = tx.last_insert_rowid();
                f.execute(params![id, doc.kind, doc.title, doc.tags, doc.body]).unwrap();
                for (t, a) in &doc.links {
                    e.execute(params![id, t, a]).unwrap();
                }
            }
        }
        tx.commit().unwrap();
        let t_rows = t0.elapsed();
        let t1 = Instant::now();
        self.c.execute_batch("CREATE INDEX edges_target ON edges(target); CREATE INDEX edges_src ON edges(src); CREATE INDEX docs_mtime ON docs(mtime);").unwrap();
        let t_idx = t1.elapsed();
        let t2 = Instant::now();
        self.c.execute_batch("INSERT INTO fts(fts) VALUES('optimize'); PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
        vec![("insert rows+fts", t_rows), ("btree indexes", t_idx), ("fts optimize+ckpt", t2.elapsed())]
    }
    fn upsert(&mut self, doc: &Doc, mtime: i64) {
        let tx = self.c.transaction().unwrap();
        let old: Option<i64> = tx.query_row("SELECT id FROM docs WHERE path=?1", [&doc.path], |r| r.get(0)).ok();
        let id = match old {
            Some(id) => {
                tx.execute("DELETE FROM fts WHERE rowid=?1", [id]).unwrap();
                tx.execute("DELETE FROM edges WHERE src=?1", [id]).unwrap();
                tx.execute("UPDATE docs SET kind=?2,title=?3,mtime=?4 WHERE id=?1", params![id, doc.kind, doc.title, mtime]).unwrap();
                id
            }
            None => {
                tx.execute("INSERT INTO docs(path,kind,title,mtime) VALUES(?1,?2,?3,?4)", params![doc.path, doc.kind, doc.title, mtime]).unwrap();
                tx.last_insert_rowid()
            }
        };
        tx.execute("INSERT INTO fts(rowid,kind,title,tags,body) VALUES(?1,?2,?3,?4,?5)", params![id, doc.kind, doc.title, doc.tags, doc.body]).unwrap();
        for (t, a) in &doc.links {
            tx.execute("INSERT INTO edges(src,target,anchor) VALUES(?1,?2,?3)", params![id, t, a]).unwrap();
        }
        tx.commit().unwrap();
    }
    fn word(&self, w: &str) -> Vec<String> {
        self.m(&format!("\"{w}\""))
    }
    fn prefix(&self, p: &str) -> Vec<String> {
        self.m(&format!("\"{p}\"*"))
    }
    fn phrase(&self, words: &str) -> Vec<String> {
        self.m(&format!("\"{words}\""))
    }
    fn field(&self, field: &str, value: &str) -> Vec<String> {
        // `tags:tui keyboard` -> tags:tui AND keyboard
        let mut it = value.split(' ');
        let first = it.next().unwrap();
        let rest: Vec<&str> = it.collect();
        let mut q = format!("{field}:\"{first}\"");
        for r in rest {
            q.push_str(&format!(" AND \"{r}\""));
        }
        self.m(&q)
    }
    fn backlinks(&self, target: &str) -> Vec<String> {
        let mut s = self.c.prepare_cached("SELECT DISTINCT d.path FROM edges e JOIN docs d ON d.id=e.src WHERE e.target=?1 ORDER BY d.path LIMIT 20").unwrap();
        s.query_map([target], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
    }
    fn recent(&self, w: &str) -> Vec<String> {
        let mut s = self.c.prepare_cached("SELECT d.path FROM fts JOIN docs d ON d.id=fts.rowid WHERE fts MATCH ?1 ORDER BY d.mtime DESC LIMIT 20").unwrap();
        s.query_map([format!("\"{w}\"")], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
    }
    fn all_mtimes(&self) -> Vec<(String, i64)> {
        let mut s = self.c.prepare("SELECT path, mtime FROM docs").unwrap();
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().map(|r| r.unwrap()).collect()
    }
    fn upsert_many(&mut self, docs: &[(Doc, i64)]) {
        let tx = self.c.transaction().unwrap();
        for (doc, mtime) in docs {
            let old: Option<i64> = tx.query_row("SELECT id FROM docs WHERE path=?1", [&doc.path], |r| r.get(0)).ok();
            let id = match old {
                Some(id) => {
                    tx.execute("DELETE FROM fts WHERE rowid=?1", [id]).unwrap();
                    tx.execute("DELETE FROM edges WHERE src=?1", [id]).unwrap();
                    tx.execute("UPDATE docs SET kind=?2,title=?3,mtime=?4 WHERE id=?1", params![id, doc.kind, doc.title, mtime]).unwrap();
                    id
                }
                None => {
                    tx.execute("INSERT INTO docs(path,kind,title,mtime) VALUES(?1,?2,?3,?4)", params![doc.path, doc.kind, doc.title, mtime]).unwrap();
                    tx.last_insert_rowid()
                }
            };
            tx.execute("INSERT INTO fts(rowid,kind,title,tags,body) VALUES(?1,?2,?3,?4,?5)", params![id, doc.kind, doc.title, doc.tags, doc.body]).unwrap();
            for (t, a) in &doc.links {
                tx.execute("INSERT INTO edges(src,target,anchor) VALUES(?1,?2,?3)", params![id, t, a]).unwrap();
            }
        }
        tx.commit().unwrap();
    }
    fn count(&self) -> usize {
        self.c.query_row("SELECT count(*) FROM docs", [], |r| r.get::<_, i64>(0)).unwrap() as usize
    }
}

impl Sq {
    fn m(&self, q: &str) -> Vec<String> {
        // ranked by bm25 (default FTS5 rank), top 20
        let mut s = self.c.prepare_cached("SELECT d.path FROM fts JOIN docs d ON d.id=fts.rowid WHERE fts MATCH ?1 ORDER BY fts.rank LIMIT 20").unwrap();
        s.query_map([q], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
    }
}

fn main() {
    harness::main::<Sq>();
}
