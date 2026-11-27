//! gen <real-farm> <out-dir> <n-yaks>: synthetic farm of n yak files with a git history.
use common::gen_util::Rng;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let real = std::path::Path::new(&a[1]);
    let out = std::path::Path::new(&a[2]);
    let n: usize = a[3].parse().unwrap();
    // Sample text from the real farm so word frequencies and phrases are natural.
    let docs = common::scan(real);
    let bodies: Vec<Vec<&str>> = docs.iter().filter(|d| d.kind == "yaks" || d.kind == "doks").map(|d| d.body.split_whitespace().collect()).filter(|w: &Vec<&str>| w.len() > 50).collect();
    let titles: Vec<&str> = docs.iter().filter(|d| d.kind == "yaks").map(|d| d.title.as_str()).collect();
    let tags = ["cli", "tui", "doks", "ui", "skills", "git", "delta", "perf", "rust", "docs", "bug", "design"];
    let mut r = Rng(0x9e3779b97f4a7c15);
    let ids: Vec<String> = (0..n).map(|i| format!("yaks-{:04x}", (i * 2654435761usize >> 4) & 0xffff ^ (i & 0xf))).collect();
    // ids must be unique: fall back to index-derived suffix when collision
    let mut seen = std::collections::HashSet::new();
    let ids: Vec<String> = ids.into_iter().enumerate().map(|(i, s)| if seen.insert(s.clone()) { s } else { let s2 = format!("yaks-{:05x}", i); seen.insert(s2.clone()); s2 }).collect();

    let _ = std::fs::remove_dir_all(out);
    std::fs::create_dir_all(out).unwrap();
    let ok = |c: &mut Command| assert!(c.current_dir(out).stdout(Stdio::null()).status().unwrap().success());
    ok(Command::new("git").args(["init", "-q", "-b", "yaks"]));
    let mut fi = Command::new("git").current_dir(out).args(["fast-import", "--quiet"]).stdin(Stdio::piped()).spawn().unwrap();
    let mut w = std::io::BufWriter::new(fi.stdin.take().unwrap());

    let mut content: Vec<String> = Vec::with_capacity(n);
    let mut gen_body = |r: &mut Rng, target: usize| -> String {
        let mut s = String::new();
        while s.len() < target {
            let b = &bodies[r.below(bodies.len())];
            let len = 8 + r.below(25);
            let off = r.below(b.len().saturating_sub(len).max(1));
            let run = b[off..(off + len).min(b.len())].join(" ");
            if r.below(7) == 0 { s.push_str("\n\n"); } else if r.below(9) == 0 { s.push_str("\n- "); } else { s.push(' '); }
            s.push_str(&run);
        }
        s
    };
    let mk = |r: &mut Rng, i: usize, rev: usize, body_fn: &mut dyn FnMut(&mut Rng, usize) -> String| -> String {
        let title = titles[r.below(titles.len())];
        let mut t = String::new();
        let nt = 1 + r.below(3);
        let mut tg: Vec<&str> = (0..nt).map(|_| tags[r.below(tags.len())]).collect();
        tg.sort(); tg.dedup();
        t.push_str(&format!("---\nid: {}\ntitle: {}\nstatus: {}\ntype: task\npriority: {}\ncreated: '2026-0{}-{:02}T10:00:00Z'\nupdated: '2026-11-27T02:38:57Z'\n", ids[i], title.replace(':', " -"), ["hairy", "shaving", "shorn", "dead"][r.below(4)], 1 + r.below(4), 1 + r.below(9), 1 + r.below(28)));
        if r.below(3) == 0 { t.push_str(&format!("parent: {}\n", ids[r.below(n)])); }
        t.push_str("tags:\n");
        for x in &tg { t.push_str(&format!("- {x}\n")); }
        t.push_str("---\n");
        let size = 1000 + r.below(2000);
        let mut body = body_fn(r, size);
        // links: 2-5 yak links, 0-2 doks, 0-2 urls
        body.push_str("\n\n");
        for _ in 0..(2 + r.below(4)) { let j = r.below(n); body.push_str(&format!("See [{}](/yaks/yaks/{}.md) and ", ids[j], ids[j])); }
        for _ in 0..r.below(3) { body.push_str(&format!("[dok](/doks/design/dok-{}.md#section-{}) ", r.below(40), r.below(9))); }
        for _ in 0..r.below(3) { body.push_str(&format!("[issue](https://github.com/example/repo/issues/{}) ", 1 + r.below(5000))); }
        if rev > 0 { body.push_str(&format!("\n\n#### rev {rev}\nedited again with `issues/{}` and {}.\n", 100 + r.below(900), ids[r.below(n)])); }
        t.push_str(&body);
        t.push('\n');
        t
    };
    for i in 0..n { let c = mk(&mut r, i, 0, &mut gen_body); content.push(c); }

    // history: ~100 days; each yak created in one commit (batches of 1..5 files),
    // then ~2.5 further touch commits per yak on average; one mechanical migration commit.
    let mut mark = 0usize;
    let mut t0: i64 = 1_760_000_000;
    let mut commit = |w: &mut dyn Write, t: i64, msg: &str, files: &[(usize, &str)]| {
        mark += 1;
        writeln!(w, "commit refs/heads/yaks\nmark :{mark}\ncommitter t <t@t> {t} +0000\ndata {}\n{msg}", msg.len()).unwrap();
        for (i, c) in files {
            writeln!(w, "M 100644 inline yaks/yaks/{}.md\ndata {}\n{c}", ids[*i], c.len()).unwrap();
        }
        writeln!(w).unwrap();
    };
    let mut i = 0;
    while i < n {
        let k = (1 + r.below(5)).min(n - i);
        let files: Vec<(usize, &str)> = (i..i + k).map(|j| (j, content[j].as_str())).collect();
        t0 += 60 + r.below(600) as i64;
        commit(&mut w, t0, &format!("yaks: created {}", ids[i]), &files);
        i += k;
    }
    let mid = n * 3 / 2;
    let mut touched = 0;
    let mut step = 0;
    while touched < n * 5 / 2 {
        let j = r.below(n);
        let newc = mk(&mut r, j, 1 + step % 7, &mut gen_body);
        content[j] = newc;
        t0 += 30 + r.below(500) as i64;
        let c = content[j].clone();
        commit(&mut w, t0, &format!("yaks: updated {}", ids[j]), &[(j, c.as_str())]);
        touched += 1;
        step += 1;
        if touched == mid {
            // mechanical migration touching every file: whitespace-only change
            let files: Vec<(usize, String)> = (0..n).map(|j| (j, format!("{}\n", content[j]))).collect();
            let refs: Vec<(usize, &str)> = files.iter().map(|(j, c)| (*j, c.as_str())).collect();
            t0 += 3600;
            commit(&mut w, t0, "yaks: migrate layout\n\nMechanical: true", &refs);
            for (j, c) in files { content[j] = c; }
        }
    }
    w.flush().unwrap();
    drop(w);
    assert!(fi.wait().unwrap().success());
    ok(Command::new("git").args(["checkout", "-q", "-f", "yaks"]));
    // restore plausible fs mtimes? leave them: fresh checkout, like a clone.
    // doks stub so `doks/design` links have targets-ish
    println!("generated {} yaks, {} commits", n, mark);
}
