---
id: yak-6f9e.8
title: Suppress tab focus halo
type: bug
priority: 3
created: '2026-04-18T00:15:17Z'
updated: '2026-04-18T01:10:02Z'
commit: ab70a7e
---

Pico's default focus styling adds an ugly outline/halo to the tab buttons. Suppress it with outline:none or a subtler focus indicator.

---
▸ 2026-04-18T00:42:48Z
Fixed in v2 polish pass.

---
▸ 2026-04-18T01:09:49Z
Root cause: pico redefines --pico-color to #fff inside button elements. Fixed by using hardcoded #1b2832 instead of var(--pico-color). Also suppressed box-shadow (pico's focus indicator) with !important.

![Tabs visible, no halo, no wrapping](artifacts/yak-6f9e.8/tabs-fixed.png)
