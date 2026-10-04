# Public wire layout migration (local Week 14 patch)

Base revision: `4b7caace1f2128e454c8d0fe50cac1ec46b1e272`.

R1CS wires now contain constant one, public returns, public parameters,
private parameters, and remaining original witnesses, followed by lowering-created
wires. Within each group, ACIR witness indices are ascending. This is an ACIR
ordering contract, not a claim that arbitrary compiler versions preserve source
or ABI flattening order. Input/return group overlap is rejected, including public
returns aliasing private or public inputs. No duplicate output wire is invented.

`R1csSystem.acir_wire_map` carries the original witness-to-wire mapping used by
lowering. JSON requires this field; historical JSON without it must be regenerated.
Binary R1CS/WTNS encodings are unchanged, but circuit wire identities change:
regenerate development setup keys and proofs. Never pair old keys with new R1CS.

`WitnessArtifacts.witness_vector`, `witness.bin`, witness maps and
`compile_r1cs_with_witness` remain ACIR-indexed (one, w0, w1, ...).
`materialize_witness` consumes that representation and returns R1CS wire order.
Use `is_satisfied_with_full_witness` for an already materialized assignment.
`instance_witness_value` intentionally still reads ACIR index plus one.

The CLI `witness` command now names its raw diagnostic `acir.wtns`, rather than
`witness.wtns`, to distinguish it from the proving assignment. Use strict `interop`
for the paired `circuit.r1cs` and fully materialized `witness.wtns`. The low-level
WTNS writer serializes its supplied vector; it does not independently permute it.

The toolkit preview remains limited to scalar Field parameters without returns.
Arrays, structs and returns need compiler/ABI differential evidence before a
broader application compatibility claim. Synthetic ACIR ordering tests alone
are not proof of compiler compatibility. A layout correction is not an audit.
