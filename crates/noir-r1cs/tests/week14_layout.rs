use acir::AcirField;
use acir::{
    circuit::{Circuit, Opcode, Program, PublicInputs},
    native_types::{Expression, Witness},
    FieldElement,
};
use noir_r1cs::compile_r1cs;
use std::collections::BTreeSet;

fn square(public_first: bool) -> Program<FieldElement> {
    let (x, y) = if public_first { (1, 0) } else { (0, 1) };
    Program {
        functions: vec![Circuit {
            current_witness_index: 1,
            private_parameters: BTreeSet::from([Witness(x)]),
            public_parameters: PublicInputs(BTreeSet::from([Witness(y)])),
            opcodes: vec![Opcode::AssertZero(Expression {
                mul_terms: vec![(FieldElement::one(), Witness(x), Witness(x))],
                linear_combinations: vec![(-FieldElement::one(), Witness(y))],
                q_c: FieldElement::zero(),
            })],
            ..Circuit::default()
        }],
        unconstrained_functions: vec![],
    }
}

#[test]
fn private_first_exposes_declared_public_value() {
    let system = compile_r1cs(&square(false)).unwrap();
    let acir = [1u128, 7, 49].map(FieldElement::from);
    let full = system.materialize_witness(&acir).unwrap();
    assert_eq!(full[1], FieldElement::from(49u128));
    assert_eq!(full[2], FieldElement::from(7u128));
}

#[test]
fn public_first_control_preserves_declared_public_value() {
    let system = compile_r1cs(&square(true)).unwrap();
    let acir = [1u128, 49, 7].map(FieldElement::from);
    let full = system.materialize_witness(&acir).unwrap();
    assert_eq!(full[1], FieldElement::from(49u128));
    assert_eq!(full[2], FieldElement::from(7u128));
}

#[test]
fn invalid_square_and_wrong_constant_are_rejected() {
    let system = compile_r1cs(&square(false)).unwrap();
    assert!(system
        .materialize_witness(&[1u128, 7, 50].map(FieldElement::from))
        .is_none());
    assert!(system
        .materialize_witness(&[0u128, 7, 49].map(FieldElement::from))
        .is_none());
    let mut full = system
        .materialize_witness(&[1u128, 7, 49].map(FieldElement::from))
        .unwrap();
    assert!(system.is_satisfied_with_full_witness(&full));
    full[1] = FieldElement::from(7u128);
    assert!(!system.is_satisfied_with_full_witness(&full));
}

#[test]
fn output_public_private_internal_order_and_json_roundtrip() {
    let program = Program {
        functions: vec![Circuit {
            current_witness_index: 5,
            return_values: PublicInputs(BTreeSet::from([Witness(4)])),
            public_parameters: PublicInputs(BTreeSet::from([Witness(1), Witness(3)])),
            private_parameters: BTreeSet::from([Witness(0), Witness(2)]),
            opcodes: vec![],
            ..Circuit::default()
        }],
        unconstrained_functions: vec![],
    };
    let system = compile_r1cs(&program).unwrap();
    let restored: noir_r1cs::R1csSystem =
        serde_json::from_str(&serde_json::to_string(&system).unwrap()).unwrap();
    let acir = [1u128, 7, 11, 13, 17, 19, 23].map(FieldElement::from);
    let expected = [1u128, 19, 11, 17, 7, 13, 23].map(FieldElement::from);
    assert_eq!(system.materialize_witness(&acir).unwrap(), expected);
    assert_eq!(restored.materialize_witness(&acir).unwrap(), expected);
}

#[test]
fn overlapping_return_or_input_classifications_are_rejected() {
    for overlap in [0, 1] {
        let mut program = square(false);
        program.functions[0].return_values = PublicInputs(BTreeSet::from([Witness(overlap)]));
        assert!(compile_r1cs(&program)
            .unwrap_err()
            .to_string()
            .contains("overlapping input/return"));
    }
    let mut program = square(false);
    program.functions[0].private_parameters.insert(Witness(1));
    assert!(compile_r1cs(&program)
        .unwrap_err()
        .to_string()
        .contains("overlapping input/return"));
}

#[test]
fn zero_public_inputs_and_out_of_range_metadata() {
    let mut program = square(false);
    program.functions[0].public_parameters = PublicInputs(BTreeSet::new());
    program.functions[0].private_parameters.insert(Witness(1));
    let system = compile_r1cs(&program).unwrap();
    assert_eq!(system.n_public_inputs, 0);
    assert!(system.is_satisfied(&[1u128, 7, 49].map(FieldElement::from)));
    program.functions[0].public_parameters = PublicInputs(BTreeSet::from([Witness(9)]));
    assert!(compile_r1cs(&program).is_err());
}

#[test]
fn beta18_compiler_fixtures_match_independent_public_expectations() {
    use noir_acir::{AbiVisibility, Artifact};
    use noir_r1cs::compile_r1cs_with_witness;
    use noir_witness::generate_witness_from_json_str;
    use std::{fs, path::Path};
    for case in [
        "square-root",
        "square-root-public-first",
        "interleaved",
        "zero_public",
        "proof-bound-capsule",
    ] {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-vectors/week14")
            .join(case);
        let artifact =
            Artifact::from_json_bytes(&fs::read(dir.join("artifact.json")).unwrap()).unwrap();
        assert_eq!(
            artifact.noir_version.as_deref(),
            Some("1.0.0-beta.18+99bb8b5cf33d7669adbdef096b12d80f30b4c0c9")
        );
        let inputs_text = fs::read_to_string(dir.join("inputs.json")).unwrap();
        let inputs: serde_json::Value = serde_json::from_str(&inputs_text).unwrap();
        let expected: Vec<String> =
            serde_json::from_slice(&fs::read(dir.join("expected-public.json")).unwrap()).unwrap();
        // Independent source-order ABI comparison: these fixtures use scalar Fields.
        let abi_public: Vec<String> = artifact
            .abi
            .parameters
            .iter()
            .filter(|p| p.visibility == AbiVisibility::Public)
            .map(|p| inputs[&p.name].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(abi_public, expected, "{case}: source ABI expectation");
        let witness = generate_witness_from_json_str(&artifact, &inputs_text).unwrap();
        let system = compile_r1cs_with_witness(&artifact.program, &witness.witness_vector).unwrap();
        let full = system.materialize_witness(&witness.witness_vector).unwrap();
        assert_eq!(system.n_public_inputs as usize, expected.len());
        assert_eq!(system.n_public_outputs, 0);
        let expected_fields: Vec<_> = expected
            .iter()
            .map(|s| FieldElement::try_from_str(s).unwrap())
            .collect();
        assert_eq!(
            &full[1..1 + expected.len()],
            expected_fields,
            "{case}: public wire order"
        );
        assert!(system.is_satisfied_with_full_witness(&full));
        if !expected.is_empty() {
            let mut wrong = full.clone();
            wrong[1] += FieldElement::one();
            assert!(
                !system.is_satisfied_with_full_witness(&wrong),
                "{case}: changed public wire"
            );
        }
        if case == "interleaved" {
            let mut wrong = full.clone();
            wrong.swap(1, 2);
            assert!(!system.is_satisfied_with_full_witness(&wrong));
        }
    }
}
