//! tantivy prototype: one index holds text postings (positions), link edges (a multi-valued
//! raw `links` field, so backlinks are a term query), mtime (fast i64 field).
//! Optional identifier-aware tokenizer (env IDTOK=1).
use common::harness::{self, Backend};
use common::Doc;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, PhraseQuery, Query, RegexQuery, TermQuery};
use tantivy::schema::*;
use tantivy::tokenizer::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, Term};

mod idtok;

pub struct Tv {
    index: Index,
    reader: IndexReader,
    writer: Option<IndexWriter>,
    f_path: Field,
    f_kind: Field,
    f_title: Field,
    f_tags: Field,
    f_body: Field,
    f_links: Field,
    f_mtime: Field,
}

fn schema() -> Schema {
    let mut b = Schema::builder();
    let text = |name: &str, b: &mut SchemaBuilder, stored: bool| {
        let opts = TextOptions::default().set_indexing_options(TextFieldIndexing::default().set_tokenizer("yaks").set_index_option(IndexRecordOption::WithFreqsAndPositions));
        b.add_text_field(name, if stored { opts.set_stored() } else { opts })
    };
    b.add_text_field("path", STRING | STORED | FAST);
    b.add_text_field("kind", STRING);
    text("title", &mut b, true);
    text("tags", &mut b, false);
    text("body", &mut b, false);
    // edges: one raw term per link ("target" and "target#anchor"); stored so the edge list can be read back
    b.add_text_field("links", STRING | STORED);
    b.add_i64_field("mtime", FAST | STORED | INDEXED);
    b.build()
}

fn analyzer() -> TextAnalyzer {
    if std::env::var("IDTOK").is_ok() {
        TextAnalyzer::builder(idtok::IdentTokenizer::default()).filter(LowerCaser).filter(AsciiFoldingFilter).build()
    } else {
        TextAnalyzer::builder(SimpleTokenizer::default()).filter(LowerCaser).filter(AsciiFoldingFilter).build()
    }
}

fn wrap(index: Index) -> Tv {
    index.tokenizers().register("yaks", analyzer());
    let s = index.schema();
    let reader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into().unwrap();
    Tv {
        reader,
        writer: None,
        f_path: s.get_field("path").unwrap(),
        f_kind: s.get_field("kind").unwrap(),
        f_title: s.get_field("title").unwrap(),
        f_tags: s.get_field("tags").unwrap(),
        f_body: s.get_field("body").unwrap(),
        f_links: s.get_field("links").unwrap(),
        f_mtime: s.get_field("mtime").unwrap(),
        index,
    }
}

impl Tv {
    fn add(&self, w: &IndexWriter, d: &Doc, mtime: i64) {
        let mut t = doc!(
            self.f_path => d.path.as_str(), self.f_kind => d.kind.as_str(), self.f_title => d.title.as_str(),
            self.f_tags => d.tags.as_str(), self.f_body => d.body.as_str(), self.f_mtime => mtime
        );
        for (target, anchor) in &d.links {
            t.add_text(self.f_links, target);
            if !anchor.is_empty() {
                t.add_text(self.f_links, format!("{target}#{anchor}"));
            }
        }
        w.add_document(t).unwrap();
    }
    fn paths(&self, q: &dyn Query) -> Vec<String> {
        let s = self.reader.searcher();
        let top = s.search(q, &TopDocs::with_limit(20).order_by_score()).unwrap();
        top.into_iter().map(|(_, a)| self.path_of(&s, a)).collect()
    }
    fn path_of(&self, s: &tantivy::Searcher, a: tantivy::DocAddress) -> String {
        let d: TantivyDocument = s.doc(a).unwrap();
        d.get_first(self.f_path).and_then(|v| v.as_str()).unwrap().to_string()
    }
    fn any_field(&self, mk: impl Fn(Field) -> Box<dyn Query>) -> Box<dyn Query> {
        Box::new(BooleanQuery::new(vec![self.f_title, self.f_tags, self.f_body].into_iter().map(|f| (Occur::Should, mk(f))).collect()))
    }
    fn term(&self, f: Field, w: &str) -> Box<dyn Query> {
        Box::new(TermQuery::new(Term::from_field_text(f, w), IndexRecordOption::WithFreqs))
    }
}

impl Backend for Tv {
    const NAME: &'static str = "tantivy";
    fn create(dir: &Path) -> Self {
        let mut t = wrap(Index::create_in_dir(dir, schema()).unwrap());
        let nt: usize = std::env::var("TV_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        t.writer = Some(if nt == 0 { t.index.writer(256_000_000).unwrap() } else { t.index.writer_with_num_threads(nt, 50_000_000 * nt).unwrap() });
        t
    }
    fn open(dir: &Path) -> Self {
        wrap(Index::open_in_dir(dir).unwrap())
    }
    fn build(&mut self, docs: &[Doc], times: &HashMap<String, i64>) -> Vec<(&'static str, Duration)> {
        let t0 = Instant::now();
        let mut w = self.writer.take().unwrap();
        for d in docs {
            self.add(&w, d, times[&d.path]);
        }
        let t_add = t0.elapsed();
        let t1 = Instant::now();
        w.commit().unwrap();
        let t_commit = t1.elapsed();
        let t2 = Instant::now();
        // let the default merge policy finish, then GC the replaced segment files
        w.wait_merging_threads().unwrap();
        let w2: IndexWriter = self.index.writer_with_num_threads(1, 15_000_000).unwrap();
        let _ = futures_lite_block(w2.garbage_collect_files());
        drop(w2);
        eprintln!("segments after: {}", self.index.searchable_segment_ids().unwrap().len());
        vec![("add docs", t_add), ("commit", t_commit), ("merge wait + gc", t2.elapsed())]
    }
    fn upsert(&mut self, d: &Doc, mtime: i64) {
        let mut w: IndexWriter = self.index.writer_with_num_threads(1, 15_000_000).unwrap();
        w.delete_term(Term::from_field_text(self.f_path, &d.path));
        self.add(&w, d, mtime);
        w.commit().unwrap();
        self.reader.reload().unwrap();
        // writer dropped here: drop waits for merge threads
    }
    fn word(&self, w: &str) -> Vec<String> {
        // run the query word through the field analyzer; for an identifier use its whole-identifier token
        let mut an = self.index.tokenizers().get("yaks").unwrap();
        let mut best: Option<Token> = None;
        let mut ts = an.token_stream(w);
        while ts.advance() {
            let t = ts.token().clone();
            if best.as_ref().map_or(true, |b| t.position_length > b.position_length) {
                best = Some(t);
            }
        }
        let w = best.map(|t| t.text).unwrap_or_else(|| w.to_string());
        let q = self.any_field(|f| self.term(f, &w));
        self.paths(&*q)
    }
    fn prefix(&self, p: &str) -> Vec<String> {
        let q = self.any_field(|f| Box::new(RegexQuery::from_pattern(&format!("{}.*", p), f).unwrap()));
        self.paths(&*q)
    }
    fn phrase(&self, words: &str) -> Vec<String> {
        let ws: Vec<&str> = words.split(' ').collect();
        let q = self.any_field(|f| Box::new(PhraseQuery::new(ws.iter().map(|w| Term::from_field_text(f, w)).collect())));
        self.paths(&*q)
    }
    fn field(&self, field: &str, value: &str) -> Vec<String> {
        let f = self.index.schema().get_field(field).unwrap();
        let mut it = value.split(' ');
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = vec![(Occur::Must, self.term(f, it.next().unwrap()))];
        for r in it {
            clauses.push((Occur::Must, self.any_field(|f| self.term(f, r))));
        }
        self.paths(&BooleanQuery::new(clauses))
    }
    fn backlinks(&self, target: &str) -> Vec<String> {
        let q = TermQuery::new(Term::from_field_text(self.f_links, target), IndexRecordOption::Basic);
        let s = self.reader.searcher();
        // no score needed: sort by path via stored value after collecting top-N by doc order
        let top = s.search(&q, &TopDocs::with_limit(20).order_by_score()).unwrap();
        top.into_iter().map(|(_, a)| self.path_of(&s, a)).collect()
    }
    fn recent(&self, w: &str) -> Vec<String> {
        let q = self.any_field(|f| self.term(f, w));
        let s = self.reader.searcher();
        let top = s.search(&*q, &TopDocs::with_limit(20).order_by_fast_field::<i64>("mtime", tantivy::Order::Desc)).unwrap();
        top.into_iter().map(|(_, a)| self.path_of(&s, a)).collect()
    }
    fn all_mtimes(&self) -> Vec<(String, i64)> {
        let s = self.reader.searcher();
        let mut out = Vec::new();
        for seg in s.segment_readers() {
            let ff = seg.fast_fields();
            let paths = ff.str("path").unwrap().unwrap();
            let mt = ff.i64("mtime").unwrap();
            let alive = seg.alive_bitset();
            let mut buf = String::new();
            for doc in 0..seg.max_doc() {
                if alive.map_or(false, |b| !b.is_alive(doc)) {
                    continue;
                }
                let ord = paths.term_ords(doc).next().unwrap();
                buf.clear();
                paths.ord_to_str(ord, &mut buf).unwrap();
                out.push((buf.clone(), mt.first(doc).unwrap()));
            }
        }
        out
    }
    fn upsert_many(&mut self, docs: &[(Doc, i64)]) {
        let mut w: IndexWriter = self.index.writer_with_num_threads(1, 50_000_000).unwrap();
        for (d, m) in docs {
            w.delete_term(Term::from_field_text(self.f_path, &d.path));
            self.add(&w, d, *m);
        }
        w.commit().unwrap();
        self.reader.reload().unwrap();
    }
    fn count(&self) -> usize {
        self.reader.searcher().num_docs() as usize
    }
}

fn futures_lite_block<F: std::future::Future>(f: F) -> F::Output {
    // tantivy's merge future is a plain oneshot receiver; block on it with a tiny executor
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    fn noop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VT)
    }
    static VT: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VT)) };
    let mut cx = Context::from_waker(&waker);
    let mut f = Box::pin(f);
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn main() {
    harness::main::<Tv>();
}
