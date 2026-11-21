---
type: Dok Type
title: Release
description: 'One release: its headlines, then its changes between <!-- yaks:changes --> and <!-- /yaks:changes -->, which `yaks release dok` rewrites from the yaks'
fields:
  version: {type: text, required: true}
  tag: text
  date: date
  range: text
  yanked: bool
---
