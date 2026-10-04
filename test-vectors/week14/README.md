# Week 14 compiler fixtures

Compiler: Nargo beta.18, noirc revision 99bb8b5cf33d7669adbdef096b12d80f30b4c0c9.
`interleaved` and `zero_public` were compiled locally with `nargo compile`.
Square-root and Capsule artifacts are copied from the retained toolkit compiler
outputs; sources and public development inputs are included for reproduction.
`expected-public.json` is an independent test oracle, not prover output.
All committed inputs are deliberately public development test values.
These scalar Field fixtures establish ordering only for this compiler profile.
Arrays/structs/Noir returns are outside this tested application subset.
