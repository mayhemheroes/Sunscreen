#![no_main]
use libfuzzer_sys::fuzz_target;
use curve25519_dalek::scalar::Scalar;
use sunscreen_math::CpuScalarVec;

fuzz_target!(|input: Vec<u64>| {
    let a: Vec<_> = input.iter().map(|x| Scalar::from(*x)).collect();
    let a_vec = CpuScalarVec::new(&a);
    let _ = a_vec.invert();
});
