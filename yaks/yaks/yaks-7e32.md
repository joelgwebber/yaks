---
id: yaks-7e32
title: 'One document model: yaks, doks, sessions and sources on one set of constructs'
status: hairy
type: design
priority: 2
created: '2026-11-27T02:09:06Z'
updated: '2026-11-27T02:09:06Z'
tags:
- design
- doks
---

Umbrella for unifying yaks, doks, sessions, sources and release doks on one set of constructs: fields and schemas, fields with behaviour, links, notes, attachments, queries and a derived index. Kinds stay registered in Rust with their own capabilities and opinionated views; the parts under them are shared.

Design: [/doks/design/one-document-model.md](/doks/design/one-document-model.md). Started with Joel on 2026-11-26 in a Delta thread; decisions are recorded in the dok as they settle.

The implementation plan (children, and the existing yaks pulled in) is filed once the open questions in the dok's section 13 are answered.
