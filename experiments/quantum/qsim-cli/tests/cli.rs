use serde_json::{json, Value};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn run(input: &[u8]) -> (i32, Value, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_qsim-cli"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let input = input.to_vec();
    let writer = thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let mut stdout = child.stdout.take().unwrap();
    let out = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let mut stderr = child.stderr.take().unwrap();
    let err = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("CLI exceeded the whole-process 30 second deadline");
        }
        thread::sleep(Duration::from_millis(5));
    };
    writer.join().unwrap();
    let bytes = out.join().unwrap();
    let stderr = String::from_utf8(err.join().unwrap()).unwrap();
    let response: Value = serde_json::from_slice(&bytes).expect("exactly one JSON response");
    (status.code().unwrap(), response, stderr)
}

fn request() -> Value {
    json!({"schema":"qsim-v0","job_id":"test","qubits":2,"gates":[],"shots":8,"seed":"0000000000000000"})
}
fn encode(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn invalid(input: &[u8]) -> Value {
    let (exit, result, stderr) = run(input);
    assert_eq!(exit, 2, "{result}");
    assert_eq!(result["status"], "error");
    assert_eq!(result["error"]["code"], "INVALID_REQUEST");
    assert!(result.get("counts").is_none());
    assert!(stderr.is_empty());
    result
}

#[test]
fn q1_07_q1_08_q1_09_exact_fixture_schema_and_repeatability() {
    for (input, expected) in [
        (
            include_bytes!("../../fixtures/bell8.json").as_slice(),
            json!({"00":5,"11":3}),
        ),
        (
            include_bytes!("../../fixtures/bell4096.json").as_slice(),
            json!({"00":2082,"11":2014}),
        ),
    ] {
        let (exit, result, stderr) = run(input);
        assert_eq!(exit, 0);
        assert!(stderr.is_empty());
        assert_eq!(result["schema"], "qsim-v0");
        assert_eq!(result["backend"], "local-statevector-v0");
        assert_eq!(result["counts"], expected);
        assert_eq!(result["status"], "completed");
        assert_eq!(result.as_object().unwrap().len(), 8);
        assert_eq!(run(input).1, result);
    }
}

#[test]
fn q1_10_invalid_dimensions_targets_and_gate_variants() {
    for qubits in [json!(0), json!(13), json!(-1), json!(1.5), json!(u64::MAX)] {
        let mut value = request();
        value["qubits"] = qubits;
        invalid(&encode(&value));
    }
    for gate in [
        json!({"op":"x","target":2}),
        json!({"op":"cx","control":0,"target":0}),
        json!({"op":"cx","target":0}),
        json!({"op":"cnot","control":0,"target":1}),
        json!({"op":"h","target":0,"control":1}),
        json!({"op":"h","target":0,"control":null}),
        json!({"op":"x","target":u64::MAX}),
        json!({"op":"cx","control":u64::MAX,"target":1}),
    ] {
        let mut value = request();
        value["gates"] = json!([gate]);
        invalid(&encode(&value));
    }
}

#[test]
fn q1_11_request_gate_shot_and_overflow_limits() {
    let (exit, response, _) = run(&vec![b' '; 262_145]);
    assert_eq!(exit, 3);
    assert_eq!(response["error"]["code"], "RESOURCE_EXHAUSTED");
    let mut value = request();
    value["gates"] = json!(vec![json!({"op":"h","target":0}); 4097]);
    let (exit, response, _) = run(&encode(&value));
    assert_eq!(exit, 3);
    assert_eq!(response["error"]["field"], "gates");
    for shots in [
        json!(0),
        json!(65_537),
        json!(u64::MAX),
        json!(-1),
        json!(0.5),
    ] {
        let mut value = request();
        value["shots"] = shots;
        invalid(&encode(&value));
    }
    invalid(br#"{"schema":"qsim-v0","job_id":"test","qubits":18446744073709551616,"gates":[],"shots":8,"seed":"0000000000000000"}"#);
}

#[test]
fn q1_12_duplicates_unknown_fields_seed_trailing_and_truncated_json() {
    let base = String::from_utf8(encode(&request())).unwrap();
    invalid(format!("{{\"qubits\":2,{}", &base[1..]).as_bytes());
    invalid(
        base.replace(
            "\"gates\":[]",
            "\"gates\":[{\"op\":\"x\",\"target\":0,\"target\":1}]",
        )
        .as_bytes(),
    );
    invalid(format!("{base} {{}}").as_bytes());
    invalid(&base.as_bytes()[..base.len() - 1]);
    let mut value = request();
    value["unexpected"] = json!("secret-must-not-echo");
    invalid(&encode(&value));
    for seed in [
        "0",
        "000000000000000G",
        "AAAAAAAAAAAAAAAA",
        "00000000000000000",
    ] {
        let mut value = request();
        value["seed"] = json!(seed);
        assert_eq!(invalid(&encode(&value))["error"]["field"], "seed");
    }
    let mut value = request();
    value.as_object_mut().unwrap().remove("shots");
    invalid(&encode(&value));
    invalid(b"");
    invalid(&[0xff]);
}

#[test]
fn top_level_array_is_rejected() {
    invalid(br#"["qsim-v0","test",2,[],8,"0000000000000000"]"#);
}

#[test]
fn array_gates_are_rejected() {
    for gate in [json!(["h", 0]), json!(["cx", 1, 0])] {
        let mut value = request();
        value["gates"] = json!([gate]);
        invalid(&encode(&value));
    }
}

#[test]
fn ids_and_schema_are_validated_without_echoing_untrusted_data() {
    for id in ["", "bad/id", "non_ascii_\u{e9}", &"x".repeat(65)] {
        let mut value = request();
        value["job_id"] = json!(id);
        assert!(invalid(&encode(&value))["job_id"].is_null());
    }
    let mut value = request();
    value["schema"] = json!("qsim-v1");
    let (exit, response, _) = run(&encode(&value));
    assert_eq!(exit, 2);
    assert_eq!(response["error"]["code"], "UNSUPPORTED_SCHEMA");
    assert_eq!(response["job_id"], "test");
}

#[test]
fn deep_input_is_rejected_and_recursion_guard_remains_enabled() {
    invalid(format!("{}0{}", "[".repeat(512), "]".repeat(512)).as_bytes());
    let mut input = String::from_utf8(encode(&request())).unwrap();
    input = input.replace(
        "\"gates\":[]",
        &format!("\"gates\":{}0{}", "[".repeat(512), "]".repeat(512)),
    );
    invalid(input.as_bytes());
}

#[test]
fn q1_16_local_binary_needs_no_dependency_fetch_or_network_endpoint() {
    let (exit, response, _) = run(include_bytes!("../../fixtures/bell8.json"));
    assert_eq!(exit, 0);
    assert_eq!(response["counts"], json!({"00":5,"11":3}));
    // This test proves standalone execution with unusable proxies, not OS network isolation.
}

#[test]
fn q1_17_maximum_json_job_finishes_or_reports_deadline() {
    let mut value = request();
    value["qubits"] = json!(12);
    value["shots"] = json!(65_536);
    value["gates"] = json!(vec![json!({"op":"h","target":0}); 4096]);
    let input = encode(&value);
    assert!(input.len() <= 262_144);
    let (exit, response, _) = run(&input);
    if exit == 4 {
        assert_eq!(response["error"]["code"], "DEADLINE_EXCEEDED");
    } else {
        assert_eq!(exit, 0);
        assert_eq!(response["shots_completed"], 65_536);
    }
}
