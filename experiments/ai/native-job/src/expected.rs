//! Host oracle: print the fixture's expected report as JSON.
use baryon_ai_native_job::{argmax, digest, run, Weights, OUTPUT};

fn main() {
    let mut logits = [0f32; OUTPUT];
    let macs = run(&Weights::new(), &mut logits).expect("fixture runs on the host");
    let bits: Vec<String> = logits.iter().map(|v| v.to_bits().to_string()).collect();
    println!(
        "{{\"macs\":{macs},\"digest\":{},\"argmax\":{:?},\"bits\":[{}]}}",
        digest(&logits),
        argmax(&logits),
        bits.join(",")
    );
}
