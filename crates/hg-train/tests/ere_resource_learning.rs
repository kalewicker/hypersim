#![cfg(feature = "ere")]
use hg_train::ere::{resource_adapter as a, resource_learning as r};
use serde_json::{json, Value};
use std::io::Read;
#[test]
fn schedule_stream_and_guards() {
    let slots = r::schedule();
    assert_eq!(slots.len(), 7440);
    assert_eq!(slots.iter().filter(|s| s.phase == "training").count(), 6400);
    assert_eq!(
        slots.iter().filter(|s| s.phase == "confirmation").count(),
        1024
    );
    assert_eq!(slots.iter().filter(|s| s.phase == "baseline").count(), 16);
    for (i, s) in slots.iter().enumerate() {
        assert_eq!(s.id, i);
        assert!((0. ..1.).contains(&s.draw));
    }
    for rep in 0..4 {
        for i in 0..800 {
            assert_eq!(slots[rep * 1600 + i].draw, slots[rep * 1600 + 800 + i].draw);
        }
    }
    let encoded = serde_json::to_vec(&slots).unwrap();
    let decoded: Vec<r::Slot> = serde_json::from_slice(&encoded).unwrap();
    assert!(
        slots == decoded,
        "draw floats must survive schedule roundtrip"
    );
    let tmp = tempfile::tempdir().unwrap();
    assert!(r::campaign(tmp.path(), 0, 0).is_err());
    assert!(!tmp.path().join("campaign").exists());
    assert!(r::campaign(tmp.path(), r::MAX_ADVANCES, u64::MAX).is_err());
    assert!(!tmp.path().join("campaign").exists());
}
#[test]
fn update_and_compression_refusals() {
    let samples: Vec<_> = (0..8)
        .map(|state| r::Sample {
            state,
            input: [1., if state % 2 == 0 { 1. } else { 0. }, 1., 1. / 6.],
            action: state % 2,
            utility: 8.,
        })
        .collect();
    let update = r::batch_update(a::PROBE, &samples).unwrap();
    assert_eq!(update.before, a::PROBE);
    for i in [1, 3, 5, 7] {
        assert_eq!(update.contributions[i], [0.; 4]);
    }
    assert!(r::batch_update(a::PROBE, &samples[..7]).is_err());
    let mut bad = samples.clone();
    bad[1].state = 0;
    assert!(r::batch_update(a::PROBE, &bad).is_err());
    bad = samples.clone();
    bad[0].utility = f64::NAN;
    assert!(r::batch_update(a::PROBE, &bad).is_err());
    bad = samples.clone();
    bad[0].action = 2;
    assert!(r::batch_update(a::PROBE, &bad).is_err());
    assert!(r::batch_update([9.; 4], &samples).is_err());
    assert!(r::clip([f64::NAN; 4]).is_err());
    assert!(r::project([f64::INFINITY; 4]).is_err());
    let (norm, scale, g) = r::clip([3., 4., 0., 0.]).unwrap();
    assert_eq!(norm, 5.);
    assert_eq!(scale, 0.2);
    assert!((g[0] - 0.6).abs() < 1e-15);
    assert_eq!(g[1], 0.8);
    assert_eq!(r::project([9., -9., 3., 0.]).unwrap(), [8., -8., 3., 0.]);
    let payload = json!({"test":[1,2,3]});
    let gzip = r::gzip_bytes(&payload).unwrap();
    assert_eq!(gzip, r::gzip_bytes(&payload).unwrap());
    let mut reader = flate2::read::GzDecoder::new(&gzip[..]);
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes).unwrap();
    assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), payload);
    assert!(r::utility(&json!({"profile":"not-native"})).is_err());
}
#[test]
#[ignore = "requires explicit saved-input preflight archive; zero native/optimizer work"]
fn saved_credit_and_arithmetic_fixture() {
    let root = std::path::PathBuf::from(
        std::env::var("ERE_RESOURCE_LEARNING_PREFLIGHT").expect("preflight path required"),
    );
    let mut samples = vec![];
    let mut cells = vec![];
    for state in 0..8 {
        let v: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("inputs/state-{state}.json"))).unwrap(),
        )
        .unwrap();
        let receipt = &v["receipt"];
        let policy = &v["policy"];
        let utility = r::utility(receipt).unwrap();
        let sample = r::Sample {
            state,
            input: serde_json::from_value(policy["input"].clone()).unwrap(),
            action: policy["sampled_action"].as_u64().unwrap() as usize,
            utility,
        };
        let gzip = r::gzip_bytes(&v).unwrap();
        std::fs::write(
            root.join(format!("compressed-state-{state}.json.gz")),
            &gzip,
        )
        .unwrap();
        cells.push(json!({"state":state,"utility":utility,"raw_bytes":std::fs::metadata(root.join(format!("inputs/state-{state}.json"))).unwrap().len(),"compressed_bytes":gzip.len()}));
        samples.push(sample);
        let mut bad = receipt.clone();
        bad["completed_steps"] = json!(31);
        assert!(r::utility(&bad).is_err());
        let mut bad = receipt.clone();
        bad["timeline"][17]["step"] = json!(18);
        assert!(r::utility(&bad).is_err());
    }
    let update = r::batch_update(a::PROBE, &samples).unwrap();
    let (norm, scale, gradient) = r::clip([3., 4., 0., 0.]).unwrap();
    let result = json!({"profile":r::PROFILE,"classification":"saved-receipt-arithmetic-only","samples":samples,"update":update,"cells":cells,"clip_probe":{"norm":norm,"scale":scale,"gradient":gradient},"project_probe":r::project([9.,-9.,3.,0.]).unwrap(),"native_advances":0,"optimizer_updates":0});
    std::fs::write(
        root.join("arithmetic.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
