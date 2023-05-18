#![no_main]
use libfuzzer_sys::fuzz_target;
use curve25519_dalek::ristretto::RistrettoPoint;
use sunscreen_math::CpuRistrettoPointVec;

fuzz_target!(|input: Vec<[u8; 64]>| {
    let a: Vec<_> = input.iter().map(|x| RistrettoPoint::from_uniform_bytes(x)).collect();
    let a_vec = CpuRistrettoPointVec::new(&a);
});