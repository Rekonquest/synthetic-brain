---
name: hybrid-specialists
description: Jesse's standing principle that every stack is a specialist for a hybrid, like brain regions - never write a stack off for losing overall; record its niche (what it is best at, with numbers) and route work to the specialist where it shines. Covers the router designs measured in HYBRID-1 (fixed by kind, per-memory "shine" by trial readback, learned/REINFORCE) and the complementary-learning-systems idea. Use whenever a stack wins in one area and loses elsewhere, when writing up any result (add a niche line), when designing a system with several stores/engines/models, when Jesse says "hybrid", "assign memories where they shine", "use stacks for what they are strongest for", or when choosing between components; also transferable to any multi-component system (caches, model routing, storage tiers).
---

# Hybrid specialists

Jesse (2026-10-24): "treat all stacks like we intended to use them for memory, a hybrid, use stacks
for what they are strongest for, so a stack that does exceptional in one area but poorly everywhere
else can be used in a hybrid set up. Like the brain, no one thing excels at everything." The field's
name: **complementary learning systems** (McClelland 1995: hippocampus fast/associative, cortex
slow/lasting, moved by sleep replay).

## Always
- Every write-up gets a **niche** line: best at what, measured how, and the hybrid role.
- A specialist that loses overall is kept if it wins somewhere.

## Routers (HYBRID-1, measured)
- **Fixed by kind** (designer's assignment): 0.758 overall vs 0.50/0.41 for single stores.
- **Shine per memory** (Jesse's idea): trial-write into each eligible store, read back the item's own
  query, keep where best (ties → cheaper), dissolve the trials. Found the assignment by itself
  (0.760) with no kinds given.
- **Learned** (REINFORCE the winning store per kind after 20% trial-routed): same result (0.758)
  with 80% fewer trial writes.
- Overflow needs a *second store with the same ability*; a router cannot invent one.

## Niche catalogue (synthetic-brain)
Thermal shape store: lasting memories that fade unless used · phase recognition: "seen before?" ·
2-bit association: partial/noisy cues, any-direction records, damage tolerance · raw/AtomDB: exact,
cheapest per byte · virial recall loop: deepest clean recall below the tipping point · ATTEND:
redundant data at depth · maximum entropy: calibrated scores without labels · Wilson reading:
drift-proof · topological ring: noise-proof bits, exact small counts · string harmonics: parallel
lanes · entangled pairs: coordination with no message · Zeno: holding a state · least action: path
planning · Noether + grid cells: recognition under rotation · mass segregation: caching ·
holographic layout: region loss · no-hair: consolidation.
