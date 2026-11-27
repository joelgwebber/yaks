use super::*;
use std::time::{Duration, Instant};

pub trait Backend: Sized {
    const NAME: &'static str;
    fn create(dir: &Path) -> Self;
    fn open(dir: &Path) -> Self;
    /// Bulk insert + commit. Returns phase timings (label, duration).
    fn build(&mut self, docs: &[Doc], times: &HashMap<String, i64>) -> Vec<(&'static str, Duration)>;
    fn upsert(&mut self, doc: &Doc, mtime: i64);
    fn word(&self, w: &str) -> Vec<String>;
    fn prefix(&self, p: &str) -> Vec<String>;
    fn phrase(&self, words: &str) -> Vec<String>;
    fn field(&self, field: &str, value: &str) -> Vec<String>;
    fn backlinks(&self, target: &str) -> Vec<String>;
    fn recent(&self, w: &str) -> Vec<String>;
    fn count(&self) -> usize;
    /// every (path, mtime): the staleness check's input
    fn all_mtimes(&self) -> Vec<(String, i64)>;
    /// upsert many files in one transaction/commit
    fn upsert_many(&mut self, docs: &[(Doc, i64)]);
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn bench<F: FnMut() -> Vec<String>>(label: &str, mut f: F) {
    let t0 = Instant::now();
    let first = f();
    let cold = t0.elapsed();
    let mut v = Vec::new();
    for _ in 0..200 {
        let t = Instant::now();
        let r = f();
        v.push(t.elapsed());
        std::hint::black_box(r);
    }
    v.sort();
    println!(
        "  {:<34} hits(top20)={:<3} first={:>8.3} ms  median={:>8.3} ms  p95={:>8.3} ms",
        label,
        first.len(),
        ms(cold),
        ms(v[100]),
        ms(v[190])
    );
}

pub fn main<B: Backend>() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    match cmd {
        // build <farm> <idxdir>
        "build" => {
            let farm = Path::new(&args[2]);
            let idx = Path::new(&args[3]);
            let _ = std::fs::remove_dir_all(idx);
            std::fs::create_dir_all(idx).unwrap();
            let t0 = Instant::now();
            let docs = scan(farm);
            let t_scan = t0.elapsed();
            let t1 = Instant::now();
            let times = git_times(farm, &docs);
            let t_git = t1.elapsed();
            let t2 = Instant::now();
            let mut b = B::create(idx);
            let phases = b.build(&docs, &times);
            let t_idx = t2.elapsed();
            let edges: usize = docs.iter().map(|d| d.links.len()).sum();
            println!("[{}] build {} docs, {} link edges, {:.1} MB body text", B::NAME, docs.len(), edges, docs.iter().map(|d| d.body.len()).sum::<usize>() as f64 / 1e6);
            println!("  scan+parse files   {:>9.1} ms", ms(t_scan));
            println!("  git log + status   {:>9.1} ms", ms(t_git));
            println!("  index build        {:>9.1} ms", ms(t_idx));
            for (l, d) in phases {
                println!("    - {:<16} {:>9.1} ms", l, ms(d));
            }
            println!("  TOTAL              {:>9.1} ms", ms(t0.elapsed()));
            drop(b);
            println!("  index size on disk {:>9.2} MB ({} files)", dir_size(idx) as f64 / 1e6, count_files(idx));
        }
        // update <farm> <idxdir> <relpath>  (reopen, re-parse one file, upsert, make searchable)
        "update" => {
            let farm = Path::new(&args[2]);
            let idx = Path::new(&args[3]);
            let rel = &args[4];
            let t0 = Instant::now();
            let mut b = B::open(idx);
            let t_open = t0.elapsed();
            let t1 = Instant::now();
            let doc = parse(farm, &farm.join(rel)).expect("parse");
            let t_parse = t1.elapsed();
            let t2 = Instant::now();
            // one-file git time: ask git for just that path
            let out = Command::new("git").current_dir(farm).args(["log", "-1", "--format=%ct", "--", rel]).output().unwrap();
            let mt = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap_or(doc.mtime_fs);
            let t_git = t2.elapsed();
            let t3 = Instant::now();
            b.upsert(&doc, mt);
            let t_up = t3.elapsed();
            let t4 = Instant::now();
            let n = b.word("zzzmarkerzzz").len();
            let t_vis = t4.elapsed();
            println!("[{}] update one file: open {:.2} ms, parse {:.2} ms, git log -1 {:.2} ms, upsert+commit {:.2} ms, first query after (hits={}) {:.2} ms; total process-in-main {:.2} ms, docs={}", B::NAME, ms(t_open), ms(t_parse), ms(t_git), ms(t_up), n, ms(t_vis), ms(t0.elapsed()), b.count());
        }
        // bench <farm> <idxdir>
        "bench" => {
            let farm = Path::new(&args[2]);
            let idx = Path::new(&args[3]);
            let t0 = Instant::now();
            let b = B::open(idx);
            let t_open = t0.elapsed();
            let docs = scan(farm);
            let mut inbound: HashMap<&str, usize> = HashMap::new();
            for d in &docs {
                for (t, _) in &d.links {
                    if !t.contains("://") && !t.starts_with("code:") {
                        *inbound.entry(t).or_default() += 1;
                    }
                }
            }
            let mut inv: Vec<_> = inbound.into_iter().collect();
            inv.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0));
            let top = inv[0].0.to_string();
            let mid = inv[inv.len() / 4].0.to_string();
            println!("[{}] open {:.3} ms; {} docs; backlink targets: top={} ({} in), p75={} ({} in)", B::NAME, ms(t_open), b.count(), top, inv[0].1, mid, inv[inv.len() / 4].1);
            let ev = |q: &str| std::env::var(q).ok();
            let rare = ev("Q_RARE").unwrap_or("rkyv".into());
            let common = ev("Q_COMMON").unwrap_or("task".into());
            bench(&format!("word rare   `{rare}`"), || b.word(&rare));
            bench(&format!("word common `{common}`"), || b.word(&common));
            bench("word `index` + sorted by mtime", || b.recent("index"));
            bench("prefix `ta*` (2 chars)", || b.prefix("ta"));
            bench("prefix `ind*` (3)", || b.prefix("ind"));
            bench("prefix `docum*` (5)", || b.prefix("docum"));
            bench("prefix `deriv*` (5, rarer)", || b.prefix("deriv"));
            bench("phrase \"derived index\"", || b.phrase("derived index"));
            bench("phrase \"the farm\" (common words)", || b.phrase("the farm"));
            bench("field tags:tui", || b.field("tags", "tui"));
            bench("field tags:tui AND word `keyboard`", || b.field("tags", "tui keyboard"));
            bench("backlinks (top target)", || b.backlinks(&top));
            bench("backlinks (p75 target)", || b.backlinks(&mid));
        }
        // batch <farm> <idxdir> <n>: staleness listing, then n-file batch update
        "batch" => {
            let farm = Path::new(&args[2]);
            let idx = Path::new(&args[3]);
            let n: usize = args[4].parse().unwrap();
            let mut b = B::open(idx);
            let t = Instant::now();
            let all = b.all_mtimes();
            println!("[{}] list all (path, mtime): {} rows in {:.2} ms", B::NAME, all.len(), ms(t.elapsed()));
            let docs = scan(farm);
            let batch: Vec<(Doc, i64)> = docs.into_iter().take(n).map(|d| { let m = d.mtime_fs; (d, m) }).collect();
            let t = Instant::now();
            b.upsert_many(&batch);
            println!("[{}] upsert {} files in one commit: {:.1} ms", B::NAME, n, ms(t.elapsed()));
        }
        // sample queries for checking correctness
        "show" => {
            let b = B::open(Path::new(&args[2]));
            let q = &args[4];
            let r = match args[3].as_str() {
                "word" => b.word(q),
                "prefix" => b.prefix(q),
                "phrase" => b.phrase(q),
                "tag" => b.field("tags", q),
                "backlinks" => b.backlinks(q),
                "recent" => b.recent(q),
                _ => vec![],
            };
            println!("[{}] {} `{}` -> {} hits", B::NAME, args[3], q, r.len());
            for p in r.iter().take(8) {
                println!("   {p}");
            }
        }
        _ => eprintln!("usage: build|update|bench|show"),
    }
}

fn count_files(p: &Path) -> usize {
    std::fs::read_dir(p).map_or(0, |rd| rd.flatten().map(|e| if e.path().is_dir() { count_files(&e.path()) } else { 1 }).sum())
}
