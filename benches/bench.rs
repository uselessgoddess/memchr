use criterion::measurement::Measurement;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, Criterion, PlottingBackend, Throughput,
};
use memchr::{gpt, libc, memchr_count, naive};
use std::time::Duration;

#[non_exhaustive]
#[derive(Copy, Clone)]
struct Input {
    corpus: &'static [u8],
    want: (u8, usize),
}

fn define<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    name: &str,
    input: Input,
    memchr: fn(u8, &[u8]) -> Option<usize>,
) {
    let (byte, count) = input.want;
    group.bench_function(name, |b| {
        b.iter(|| assert_eq!(count, memchr_count(byte, input.corpus, memchr)))
    });
}

fn all_input(c: &mut Criterion, name: &str, input: Input) {
    let mut group = c.benchmark_group(name);
    group
        .throughput(Throughput::BytesDecimal(input.corpus.len() as u64))
        .sample_size(10)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));

    define(&mut group, "naive", input, naive::memchr);
    define(&mut group, "gpt", input, gpt::memchr);
    define(&mut group, "libc", input, libc::memchr);
}

const SHERLOCK: &[u8] = b"Mr. Sherlock Holmes, who was usually very late in the mornings, save!";
const PSEUDO_HUGE: &[u8] = &[b'#'; 1024 * 128];

fn all(c: &mut Criterion) {
    all_input(c, "tiny/never", Input { corpus: SHERLOCK, want: (b'@', 0) });
    all_input(c, "tiny/rare", Input { corpus: SHERLOCK, want: (b'!', 1) });
    all_input(c, "tiny/common", Input { corpus: SHERLOCK, want: (b's', 5) });

    all_input(c, "huge/never", Input { corpus: PSEUDO_HUGE, want: (b'@', 0) });
}

criterion_group!(
    name = benches;
    config = Criterion::default().plotting_backend(PlottingBackend::Plotters);
    targets = all
);
criterion_main!(benches);
