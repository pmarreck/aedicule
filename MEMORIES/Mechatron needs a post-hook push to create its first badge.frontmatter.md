---
description: "Mechatron needs a post hook push to create its first badge."
datetime: 2026-07-18T14:38:39-04:00 # America/New_York (EDT)
tags: [mechatron, ci, continuous-integration, post, hook, push, create, badge]
---
GitHub delivers a push only to webhooks that existed when the push occurred.
For a newly Mechatron-provisioned repository, the already-pushed manifest does
not create badge JSON retroactively. After confirming the active `push` hook,
push a new tested `yolo` commit, then inspect `/badges/<repository>.json` only
after the worker completes.
