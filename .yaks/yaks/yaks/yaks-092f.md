---
id: yaks-092f
title: 'Tidy requests clean up after themselves: pickup --drop-request removes the request note'
status: shorn
type: feature
priority: 2
created: '2026-10-27T00:46:39Z'
updated: '2026-10-27T00:50:59Z'
parent: yaks-db07
labels:
- cli
---

---
▸ 2026-10-27T00:47:44Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-27T00:47:44Z [delta-lead]

Joel (2026-10-26, in the coordinating thread): a tidy should remove the request note when done, rather than add more notes.

Decision (delta-lead):

- `yaks pickup <id> --drop-request` clears `needs: agent` and removes the latest `requested: needs agent` note block. It appends no `picked up` note, and conflicts with `--note`. Git history keeps the record.
- It refuses unless the yak is awaiting an agent because of a request (`agent_why == Requested`), so it can never delete an answer.
- Both canned kinds end with it: tidy (the tidied body is the result) and summarize (the summary note is the result).
- Rejected: auto-dropping on every pickup of a request (free-text requests can carry context worth keeping), and a separate `yaks done` verb (pickup is already the only thing that clears needs).

---
▸ 2026-10-27T00:50:59Z [delta-lead]

Done (delta-lead). Pushed to the human's checkout and installed with `cargo install --path .`.

- `store::drop_latest_request` cuts the latest request note block; `Farm::pickup` takes a `drop_request` flag, and returns `NotARequest` for a yak awaiting an agent because of an answer.
- CLI: `pickup --drop-request`, which conflicts with `--note`. Both kind instructions now end with it.
- Docs: `docs/cli.md` (`### pickup` and the kind bullets), the README table, and the `yaks` skill's Requests paragraph.

Evidence:

- New tests: a farm test (an answer is refused; the request note goes, a later note stays, no picked-up note is added), which failed with the helper stubbed out; and a store test (first, middle and last block positions; no request gives `None`).
- Gate: 875 passed, 0 failed.
- Real binary on a throwaway farm: request `--kind tidy`, fix the body, `pickup --drop-request`. The diff against the original yak is only the added blank line plus `updated:`. On an answered yak `--drop-request` exits 1; with `--note` it exits 2.

---
▸ 2026-10-27T00:50:59Z [delta-lead]

moved: shaving -> shorn
