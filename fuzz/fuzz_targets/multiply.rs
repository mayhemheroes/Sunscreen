#![no_main]
use libfuzzer_sys::fuzz_target;
use sunscreen::{
    fhe_program,
    types::{bfv::Signed, Cipher},
    Compiler, Runtime,
};

#[fhe_program(scheme = "bfv")]
fn simple_multiply(a: Cipher<Signed>, b: Cipher<Signed>) -> Cipher<Signed> {
    a * b
}

fuzz_target!(|input: (i32, i32)| {
    let i0: i64 = input.0.into();
    let i1: i64 = input.1.into();
    let app = Compiler::new().fhe_program(simple_multiply).compile().unwrap();
    let runtime = Runtime::new_fhe(app.params()).unwrap();
    let (public_key, private_key) = runtime.generate_keys().unwrap();
    let a = runtime.encrypt(Signed::from(i0), &public_key).unwrap();
    let b = runtime.encrypt(Signed::from(i1), &public_key).unwrap();
    let results = runtime.run(
        app.get_fhe_program(simple_multiply).unwrap(),
        vec![a, b],
        &public_key,
    ).unwrap();
    let c: Signed = runtime.decrypt(&results[0], &private_key).unwrap();
    assert_eq!(c, (i0 * i1).into());
});