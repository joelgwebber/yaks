# Offline sync: design

## Context
R1 and R2 need entries to survive having no network and the app being killed. Today entries go straight to the API.

## Options
| Option | For | Against |
|---|---|---|
| Local-first + queue | survives app kill; works everywhere | two storage implementations |
| Retry in memory | trivial | loses entries when the app is killed |
| Third-party sync SDK | less code | lock-in, cost, web support unclear |

## Decision
Local-first + queue, owned by the shared core; platform layers only provide storage adapters. Proposed by claude, approved by sam on 2026-09-15.

## Consequences and rollback
Two storage adapters to maintain. Rollback: flush the queue, then write through to the API again.
