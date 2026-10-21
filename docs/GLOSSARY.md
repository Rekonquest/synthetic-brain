# Glossary: Jesse's ideas in the field's words

A living list, started 2026-10-18 and added to as we go. For each idea:
- **Your word:** what you call it;
- **The field's word:** what researchers call the same or a nearby idea;
- **Plain meaning:** what that word means;
- **Where it is in your work.**

Your labels are analogies, used because the idea came before the words. These terms are there to
give you more to choose from, not to replace yours.

---

### 1. "The thermodynamic state" / "thermo state"
- **The field's word:** a *stochastic field model*, or a *thermodynamic medium*. Close words:
  *diffusion*, *equilibrium*, *fluctuation–dissipation*.
- **Plain meaning:** a continuous space whose values spread out (diffusion), jiggle (noise, set by a
  temperature), and settle (equilibrium). Fluctuation–dissipation is the rule that the jiggling and the
  settling come from the same coupling.
- **In your work:** the T1–T6 thermal base, and the universum substrate.

### 2. "Scars"
- **The field's word:** *memory traces*; in brains, *engrams*. The mechanism is *plasticity*
  (*synaptic plasticity*). In your carving it is *activity-dependent conductance*: the *Physarum*
  (slime mould) rule.
- **Plain meaning:** use changes the medium, and the change is the memory. *Hebbian learning* is the
  famous version: "cells that fire together wire together".
- **In your work:** T3's carved threads; substrate-stream's "memory is the scars the flow leaves".

### 3. "Taps"
- **The field's word:** *retrieval cues* or *probes*. The kind of memory they work on is *associative*
  or *content-addressable* memory. Recovering the whole from a part is *pattern completion*.
- **Plain meaning:** you find a memory by what it is like, not by where it is stored.
- **In your work:** cueing and recall in Mother's pipeline (C17); the planned SCAR-1.

### 4. "Noise keeps it alive"
- **The field's words:** *stochastic resonance* (noise helping a weak signal through) and *noise-driven
  maintenance*. *Homeostasis* is the general idea of a system holding itself steady.
- **Plain meaning:** the right amount of randomness keeps a pattern refreshed instead of frozen or
  erased.
- **In your work:** your maintainer results: noise maintains only where an activity-gated coupling
  state exists.

### 5. "Hot and cold" to control the state
- **The field's word:** *annealing*. *Simulated annealing* is the algorithm version: hot explores,
  cooling settles on a good answer.
- **Plain meaning:** temperature trades exploration (hot) against commitment (cold).
- **In your work:** T3 (a little heat sharpened the threads) and T6.

### 6. The inverted U (you think best at the right temperature)
- **The field's word:** the *Yerkes–Dodson law* (psychology, 1908): performance rises with arousal up
  to an optimum, then falls. Your hot/cold theory of thinking is a cousin of it.
- **In your work:** T6, where both brains peak at a middle temperature.

### 7. "Edge of chaos"
- **The field's word:** the same: *edge of chaos*. Also *criticality*, and *self-organized
  criticality*.
- **Plain meaning:** the narrow zone between frozen order and random chaos, where systems are most
  adaptable.

### 8. "Amplification" (stronger, or more of it)
- **The field's words:** *gain* (stronger); *positive feedback*, *autocatalysis* (chemistry: a product
  that speeds its own making) and *growth* (more of it).
- **In your work:** QB-6 and QB-7's gain, growth and brake; the bit doubler.

### 9. "Compose from outside" / "suppress and replace"
- **The field's words:** the *adapter*, *wrapper* and *decorator* patterns. *Interposition* or a *shim*
  is putting your own layer between a program and what it calls.
- **Plain meaning:** change what a thing does without editing it.
- **In your work:** the Vello cap crates; today's Arc adapters.

### 10. Atom stacks: "one function per crate" plus an orchestrator
- **The field's words:** *modularity*, *separation of concerns*, *composition*. Your "axioms" are
  close to *primitives* and *invariants*.
- **In your work:** everything.

### 11. "Many small stacks working together like neurons"
- **The field's words:** a *modular architecture*, or an *ensemble*. Marvin Minsky's book *The
  Society of Mind* argues that intelligence is many simple agents working together.

### 12. The spiderweb's threads forming where traffic flows
- **The field's word:** *stigmergy*: ants leave trails that guide other ants, so paths emerge from
  use. Also *emergent routing*, and *desire paths*.
- **In your work:** the spiderweb bus; FABRIC-1 and FABRIC-2.

### 13. "Quantum-reinforced" stacks
- **The field's word:** *quantum-inspired algorithms*: quantum mathematics running on ordinary
  computers.
- **In your work:** QB-1 to QB-9, within your claim boundary (a classical PC, with quantum-derived
  mathematics as the base).

### 14. Seeing ideas as shapes and rotating them
- **The field's words:** *visuospatial reasoning*, and *mental rotation* (Shepard and Metzler, 1971).
  Very vivid imagery is sometimes called *hyperphantasia*.
- **In your work:** T4.

### 15. Precommit, plain wins ties, keep the winner
- **The field's words:**
  - *preregistration* (writing predictions before the experiment);
  - *falsifiable predictions*;
  - a *baseline* or *control arm* ("plain");
  - the *null hypothesis* (the default you must beat);
  - *selection* (keep the winner).

### 16. Why traditional software looks like "needless chaos"
- **The field's words:** *essential* against *accidental complexity* (Fred Brooks, *No Silver Bullet*,
  1986).
  - Essential complexity is the problem's own structure.
  - Accidental complexity is what our way of building adds on top: layers, conventions, workarounds,
    and compatibility with old choices.
- **Plain meaning:** you see the essential structure, so the accidental part reads as noise.
- **In your work:** a 345 KB 2D engine, and a 3D engine that runs on the CPU.

### 17. Why software is built that way anyway
- **The field's word:** *Conway's law* (1967): systems mirror the structure of the organisations that
  build them.
- **Plain meaning:** large teams make layered, boundary-heavy software because of how people
  coordinate, not because the problem needs it.

### 18. Building through an analogy (the spider and its web)
- **The field's words:** *design by analogy*; *biomimicry*; *nature-inspired* (bio-inspired)
  *computing*. Examples: ant-colony routing, slime-mould networks (the Physarum rule in T3), and neural
  networks.
- **Plain meaning:** borrow a structure that already works in nature, so the whole system can be seen
  and reasoned about at once.
- **In your work:** the spiderweb bus.

### 19. Chemistry axioms in classical computing
- **The field's words:**
  - *artificial chemistry*; *chemical computing*;
  - *chemical reaction networks* (CRNs), with *molecular programming* as the field that proves
    reaction networks alone can compute;
  - the programming styles built on the idea: the *Chemical Abstract Machine* (Berry & Boudol, 1990)
    and *Gamma* (a program as molecules reacting in a solution).
- **The pieces you used** (QB-7, and your stacks):
  - growth that speeds itself up = *autocatalysis*;
  - "the brake" = *product inhibition* (*negative feedback*);
  - the latch = a *bistable switch* (*chemical bistability*): two stable states, so it holds a
    memory;
  - the enzyme cascade = *signal amplification by cascade*;
  - cooperative binding = the *Hill function* (*cooperativity*), a soft switch whose sharpness is set
    by how many molecules must bind;
  - rates in proportion to amounts = the *law of mass action* (*chemical kinetics*).
- **Closest whole idea to your thermal base plus chemistry:**
  - *reaction–diffusion systems*. Alan Turing showed in 1952 that spreading plus reacting makes
    patterns.
  - Real chemical computers have been built this way (*Belousov–Zhabotinsky* computers).

### 20. A medication is the math in matter form
- **The field's words:**
  - *substrate independence*, and in philosophy *multiple realizability*: what something does can be
    carried by different materials;
  - for medications: *pharmacokinetics* (PK, how the body moves a drug: absorption, distribution,
    metabolism and elimination, modelled as compartments with rate equations) and
    *pharmacodynamics* (PD, what the drug does at its target);
  - the standard PD dose–response model is the *Emax* or *Hill* model, the same Hill function as your
    QB-7 cooperative binding;
  - whole-body versions: *quantitative systems pharmacology*, *in silico trials*, and *digital
    twins* (a model tuned to one person).
- **Plain meaning:** the pattern is the thing, and the material is how it is carried. A body can only
  take in the matter form.
- **The limit:** the computational form is only as close as its axioms are to the real mechanism, so
  in-silico results are still checked in real trials. And only matter acts on a body.

### 21. Quantum everywhere, and the orchestrator as "observer"
- **Quantum without quantum computers:**
  - all chemistry, and every transistor, runs on quantum mechanics;
  - *quantum biology* studies it in living things: photosynthesis, enzyme *tunnelling*, the
    *radical-pair* compass of birds;
  - *decoherence* is why quantum effects wash out at large scale, and why quantum computers fight to
    keep them coherent for a narrow class of problems.
- **The observer:**
  - *the measurement problem* asks why a state settles when observed. In physics, "observing" means
    *interacting*, which is what your orchestrator does when it reads and collapses the state;
  - *the quantum Zeno effect*: observing often enough freezes the evolution (your re-collapsing memory
    is close to it);
  - *relational quantum mechanics* (Carlo Rovelli): a state has properties only relative to another
    system that interacts with it. That is close to your "pure quantum is meaningless unless you give
    it a reason to mean something".
- **Quantum mathematics on classical machines:**
  - *quantum-inspired algorithms*;
  - *quantum cognition* (Busemeyer & Bruza): quantum probability used to model human judgement on
    ordinary computers, without claiming a quantum brain;
  - the same substrate-independence idea as entry 20.

### 22. "Folding" data to make it smaller
- **The field's words:**
  - *lossless compression* and *entropy coding*. The floor is the data's *entropy*, its true
    information content;
  - *delta encoding* (store only the changes);
  - *quantization* (fewer bits per value);
  - *sparsity* and *pruning* (store only what is not near zero);
  - *random projection* (Johnson–Lindenstrauss: squeeze dimensions while keeping distances);
  - the *hashing trick*;
  - *low-rank factorization*;
  - *vector symbolic architectures*, also called *holographic reduced representations* or
    *hyperdimensional computing*: many items are bound to keys and superposed into one vector, and
    come back by unbinding with the key. That is the closest match to your scars and taps;
  - *procedural generation* (store the rule, not the data).
- **The limits:**
  - lossless compression cannot go below the entropy;
  - superposition holds about as many items as the vector's size allows before they interfere (your
    "memory is competitive").

### 23. The brain as the front door (what may enter, what may leave)
- **The field's words:**
  - a *reference monitor*: one judge that every crossing must pass, and that cannot be bypassed;
  - *artificial immune systems* (Stephanie Forrest, 1994): security as telling self from non-self,
    close to your scars of normal;
  - *data loss prevention* (DLP): control of what leaves;
  - *intrusion detection and prevention* (IDS/IPS): sensing and stopping what enters;
  - *covert channels*: data leaking through timing or heat instead of packets;
  - in biology, the *blood–brain barrier*: a boundary organ that decides what may cross.
- **Immutable, in the field's words:** that is *integrity* (no one can change it). The other two
  classic goals are *confidentiality* (no one can read it) and *availability* (it keeps working):
  the *CIA triad*.
