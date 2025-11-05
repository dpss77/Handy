// Performance benchmarks for Handy

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Benchmark audio processing
fn bench_audio_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_processing");

    // Test different audio lengths
    for duration_secs in [1, 5, 10, 30].iter() {
        let sample_rate = 16000;
        let num_samples = sample_rate * duration_secs;
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| (i as f32 * 0.001).sin())
            .collect();

        group.bench_with_input(
            BenchmarkId::new("normalize", duration_secs),
            &samples,
            |b, samples| {
                b.iter(|| {
                    let max = samples.iter()
                        .map(|&s| s.abs())
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(1.0);

                    let _normalized: Vec<f32> = samples.iter()
                        .map(|&s| s / max)
                        .collect();
                    black_box(_normalized);
                });
            },
        );
    }

    group.finish();
}

// Benchmark text post-processing
fn bench_text_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_processing");

    let texts = vec![
        ("short", "hello world"),
        ("medium", "this is a medium length text with several words and some punctuation"),
        ("long", "this is a much longer text that contains many words and would represent a typical transcription result from a voice recording that lasts several seconds or even minutes and includes various types of content"),
    ];

    for (name, text) in texts.iter() {
        group.bench_with_input(
            BenchmarkId::new("capitalize_and_punctuate", name),
            text,
            |b, text| {
                b.iter(|| {
                    let mut result = text.to_string();

                    // Capitalize first letter
                    if let Some(first) = result.chars().next() {
                        result = first.to_uppercase().chain(result.chars().skip(1)).collect();
                    }

                    // Add period if missing
                    if !result.ends_with(&['.', '!', '?'][..]) {
                        result.push('.');
                    }

                    black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("normalize_whitespace", name),
            text,
            |b, text| {
                b.iter(|| {
                    let result: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

// Benchmark custom word corrections
fn bench_custom_words(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_words");

    let text = "the quik brown fox jumps over the lazy dog";
    let custom_words = vec![
        "quick".to_string(),
        "brown".to_string(),
        "lazy".to_string(),
    ];

    group.bench_function("apply_custom_words", |b| {
        b.iter(|| {
            // Simplified version without the actual implementation
            // In real benchmark, would use actual apply_custom_words
            let _result = text.to_string();
            black_box(_result);
        });
    });

    group.finish();
}

// Benchmark streaming chunk processing
fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    let chunk_sizes = vec![8000, 16000, 32000]; // 0.5s, 1s, 2s at 16kHz

    for chunk_size in chunk_sizes {
        let samples: Vec<f32> = (0..chunk_size)
            .map(|i| (i as f32 * 0.001).sin())
            .collect();

        group.bench_with_input(
            BenchmarkId::new("chunk_processing", chunk_size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    // Simulate chunk processing
                    let mut _buffer = Vec::new();
                    _buffer.extend_from_slice(samples);
                    black_box(_buffer);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_audio_processing,
    bench_text_processing,
    bench_custom_words,
    bench_streaming
);
criterion_main!(benches);
