use std::hint::black_box;
use std::time::Instant;

fn main() {
    let cases = [
        ("json", r#"{"name":"Orthos","items":[1,2,3,]}"#),
        ("yaml", "app: Orthos\nitems:\n  - one\n  - two\n"),
        ("toml", "[server\nport 8080\nmessage = \"hello\n"),
        ("xml", "<root><item>text</root>"),
        ("csv", "name,age\nAlice,30,extra\n"),
        ("ini", "[server\nhost localhost\n"),
        ("env", "NAME\nVALUE=ok\n"),
    ];
    let iterations = 200;
    let started = Instant::now();
    let mut output_bytes = 0usize;

    for _ in 0..iterations {
        for (format, content) in cases {
            let result = orthos_lib::check_format(black_box(content), black_box(format));
            output_bytes += result.errors.len();
            if let Some(corrected) = result.corrected {
                output_bytes +=
                    orthos_lib::simple_fix(black_box(&corrected), black_box(format)).len();
            }
        }
    }

    let elapsed = started.elapsed();
    println!(
        "parser benchmark: {} cases × {} iterations in {:?} (checksum {})",
        cases.len(),
        iterations,
        elapsed,
        output_bytes
    );
}
