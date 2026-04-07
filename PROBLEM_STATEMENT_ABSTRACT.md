# Problem Statement Abstract

Modern AI systems often operate behind an unclear execution boundary. A user may be told that an assistant has adopted a tool, skill, MCP, prompt set, or workflow contract, but without direct access to the active runtime, that claim is hard to verify. Behavioral resemblance is not the same as governed execution. If the user cannot inspect what was mounted, what resources were allowed, what prompts were active, and what proof artifacts were generated, then the trust relationship is fundamentally weak.

This problem gets worse when ambient capability loading is allowed. If arbitrary MCPs, tools, prompts, or session state can be mounted implicitly, then the effective runtime becomes unknowable. Authority expands invisibly. Context can drift. Tool surfaces can widen without review. The user no longer has a reliable way to determine whether the assistant is operating under a bounded contract or simply approximating one.

That has direct consequences for trust:

- claims about execution cannot be distinguished cleanly from imitation
- claims about state may rely on undeclared resources
- proofs can become narrative rather than inspectable
- the user cannot tell which parts of the system are governed and which are ambient

Governance helps by making the runtime explicit. A governed runtime declares its lanes, prompts, skills, tools, utilities, and other resources up front. It denies undeclared mounts. It requires proof for stateful or executed claims. It also requires proof of proof: a repeatable command, script, or artifact path that a human can run or inspect independently.

This changes the trust model from “please behave this way” to “show me the contract you are actually running under.” That is the key shift. A governed AI system is more trustworthy not because it claims to be safer, but because its execution boundary is inspectable, bounded, and auditable.
