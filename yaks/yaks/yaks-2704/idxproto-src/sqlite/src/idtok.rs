//! Identifier-aware FTS5 tokenizer, wrapping the built-in `unicode61`.
//!
//! For each whitespace-delimited chunk it (1) lets the wrapped tokenizer emit the
//! parts (`yaks-ca59` -> `yaks`, `ca59`), and (2) if the chunk looks like an
//! identifier (alnum runs joined by `-` or `/`, at least one joiner) also emits the
//! whole lowercased identifier as a *colocated* token at the position of the last part.
use libsqlite3_sys as ffi;
use rusqlite::Connection;
use std::ffi::{c_char, c_int, c_void, CStr, CString};

struct Tok {
    inner: ffi::fts5_tokenizer,
    inner_ctx: *mut ffi::Fts5Tokenizer,
}

struct Ctx {
    outer_ctx: *mut c_void,
    outer_cb: unsafe extern "C" fn(*mut c_void, c_int, *const c_char, c_int, c_int, c_int) -> c_int,
    base: c_int,
}

unsafe extern "C" fn fwd(ctx: *mut c_void, flags: c_int, tok: *const c_char, n: c_int, s: c_int, e: c_int) -> c_int {
    let c = &*(ctx as *const Ctx);
    (c.outer_cb)(c.outer_ctx, flags, tok, n, s + c.base, e + c.base)
}

unsafe extern "C" fn x_create(user: *mut c_void, args: *mut *const c_char, nargs: c_int, out: *mut *mut ffi::Fts5Tokenizer) -> c_int {
    // args: [<wrapped tokenizer name>, its args...]
    let api = user as *mut ffi::fts5_api;
    let mut inner: ffi::fts5_tokenizer = std::mem::zeroed();
    let (name, rest, nrest) = if nargs > 0 {
        (CStr::from_ptr(*args).to_owned(), args.add(1), nargs - 1)
    } else {
        (CString::new("unicode61").unwrap(), args, 0)
    };
    let mut ud: *mut c_void = std::ptr::null_mut();
    let rc = (*api).xFindTokenizer.unwrap()(api, name.as_ptr(), &mut ud, &mut inner);
    if rc != 0 {
        return rc;
    }
    let mut inner_ctx: *mut ffi::Fts5Tokenizer = std::ptr::null_mut();
    let rc = inner.xCreate.unwrap()(ud, rest as *mut *const c_char, nrest, &mut inner_ctx);
    if rc != 0 {
        return rc;
    }
    *out = Box::into_raw(Box::new(Tok { inner, inner_ctx })) as *mut ffi::Fts5Tokenizer;
    0
}

unsafe extern "C" fn x_delete(t: *mut ffi::Fts5Tokenizer) {
    let t = Box::from_raw(t as *mut Tok);
    if let Some(d) = t.inner.xDelete {
        d(t.inner_ctx);
    }
}

fn is_wordb(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b >= 0x80
}
fn is_joinerb(b: u8) -> bool {
    matches!(b, b'-' | b'_' | b'/')
}

/// Call the wrapped tokenizer on `bytes[from..to]`, reporting offsets relative to the whole text.
unsafe fn inner(
    t: &Tok,
    ctx: *mut c_void,
    flags: c_int,
    bytes: &[u8],
    from: usize,
    to: usize,
    cb: unsafe extern "C" fn(*mut c_void, c_int, *const c_char, c_int, c_int, c_int) -> c_int,
) -> c_int {
    let mut cx = Ctx { outer_ctx: ctx, outer_cb: cb, base: from as c_int };
    let piece = &bytes[from..to];
    t.inner.xTokenize.unwrap()(t.inner_ctx, &mut cx as *mut _ as *mut c_void, flags, piece.as_ptr() as *const c_char, piece.len() as c_int, Some(fwd))
}

unsafe extern "C" fn x_tokenize(
    t: *mut ffi::Fts5Tokenizer,
    ctx: *mut c_void,
    flags: c_int,
    text: *const c_char,
    n: c_int,
    cb: Option<unsafe extern "C" fn(*mut c_void, c_int, *const c_char, c_int, c_int, c_int) -> c_int>,
) -> c_int {
    let t = &*(t as *const Tok);
    let cb = cb.unwrap();
    let bytes = std::slice::from_raw_parts(text as *const u8, n as usize);
    let is_query = flags & 1 != 0; // FTS5_TOKENIZE_QUERY
    let mut i = 0usize;
    while i < bytes.len() {
        if !is_wordb(bytes[i]) {
            i += 1;
            continue;
        }
        // a run: word bytes joined by single `-`, `_`, `/` that sit between word bytes
        let start = i;
        let mut end = i;
        let mut seps: Vec<usize> = Vec::new(); // positions of `/` joiners
        let mut joiners = 0;
        while i < bytes.len() {
            if is_wordb(bytes[i]) {
                i += 1;
                end = i;
            } else if is_joinerb(bytes[i]) && i + 1 < bytes.len() && is_wordb(bytes[i + 1]) && i > start && is_wordb(bytes[i - 1]) {
                if bytes[i] == b'/' {
                    seps.push(i);
                }
                joiners += 1;
                i += 1;
            } else {
                break;
            }
        }
        if joiners == 0 {
            let rc = inner(t, ctx, flags, bytes, start, end, cb);
            if rc != 0 {
                return rc;
            }
            continue;
        }
        let lower = |a: usize, b: usize| bytes[a..b].iter().map(|c| c.to_ascii_lowercase()).collect::<Vec<u8>>();
        if is_query {
            // query side: the whole run is one token, so `yaks-ca59`, `issues/123`, `/yaks/yaks/yaks-ca59`
            // match the whole token indexed below (the parts stay reachable through their own queries)
            let w = lower(start, end);
            let rc = cb(ctx, 0, w.as_ptr() as *const c_char, w.len() as c_int, start as c_int, end as c_int);
            if rc != 0 {
                return rc;
            }
            continue;
        }
        // index side: per `/`-component: its parts (unicode61), plus the whole component colocated with its
        // last part when it has several; finally the whole run colocated with the last part.
        let mut bounds = vec![start];
        bounds.extend(seps.iter().map(|s| *s + 1));
        bounds.push(end + 1);
        for w in bounds.windows(2) {
            let (a, b) = (w[0], (w[1] - 1).min(end));
            let rc = inner(t, ctx, flags, bytes, a, b, cb);
            if rc != 0 {
                return rc;
            }
            let comp_has_joiner = bytes[a..b].iter().any(|c| matches!(c, b'-' | b'_'));
            if comp_has_joiner && !seps.is_empty() {
                let w = lower(a, b);
                let rc = cb(ctx, 1, w.as_ptr() as *const c_char, w.len() as c_int, a as c_int, b as c_int);
                if rc != 0 {
                    return rc;
                }
            }
        }
        let w = lower(start, end);
        // for a single component the whole run is the component: emit once
        let rc = cb(ctx, 1 /* FTS5_TOKEN_COLOCATED */, w.as_ptr() as *const c_char, w.len() as c_int, start as c_int, end as c_int);
        if rc != 0 {
            return rc;
        }
    }
    0
}

pub fn register(c: &Connection) {
    unsafe {
        let db = c.handle();
        let mut stmt: *mut ffi::sqlite3_stmt = std::ptr::null_mut();
        let sql = CString::new("SELECT fts5(?1)").unwrap();
        assert_eq!(ffi::sqlite3_prepare_v2(db, sql.as_ptr(), -1, &mut stmt, std::ptr::null_mut()), 0);
        let mut api: *mut ffi::fts5_api = std::ptr::null_mut();
        let ty = CString::new("fts5_api_ptr").unwrap();
        assert_eq!(ffi::sqlite3_bind_pointer(stmt, 1, &mut api as *mut _ as *mut c_void, ty.as_ptr(), None), 0);
        ffi::sqlite3_step(stmt);
        ffi::sqlite3_finalize(stmt);
        assert!(!api.is_null(), "no fts5_api");
        let mut tk = ffi::fts5_tokenizer { xCreate: Some(x_create), xDelete: Some(x_delete), xTokenize: Some(x_tokenize) };
        let name = CString::new("idtok").unwrap();
        let rc = (*api).xCreateTokenizer.unwrap()(api, name.as_ptr(), api as *mut c_void, &mut tk, None);
        assert_eq!(rc, 0);
    }
}
