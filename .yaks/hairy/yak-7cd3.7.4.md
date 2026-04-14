---
id: yak-7cd3.7.4
title: Extract yaktui/keys_list.py + yaktui/keys_detail.py (key dispatch)
type: task
priority: 3
created: '2026-04-14T16:28:44Z'
updated: '2026-04-14T16:28:44Z'
depends_on:
- yak-7cd3.7.3
---

Pull the two key handlers (_handle_list_key, _handle_detail_key) into their own modules. Once this lands, the App class is left with just state + the main run() loop + a handful of small helpers. Sets up step .8 nicely.
