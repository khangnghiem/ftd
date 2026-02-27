use fd_core::parser::parse_document;
use std::time::Instant;

#[test]
#[ignore] // Run manually with `cargo test --test perf_benchmark -- --nocapture --ignored`
fn benchmark_color_parsing() {
    let mut doc = String::new();
    // Generate 50,000 rectangles with hex colors
    for i in 0..50_000 {
        doc.push_str(&format!(
            "rect @r{} {{ w: 10 h: 10 fill: #{:06X} }}\n",
            i, i
        ));
    }

    let start = Instant::now();
    let _graph = parse_document(&doc).expect("parse failed");
    let duration = start.elapsed();

    println!("Parsed 50,000 colored rects in {:?}", duration);
}
