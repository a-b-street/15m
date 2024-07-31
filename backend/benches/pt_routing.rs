use backend::{MapModel, RouteRequest};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

pub fn criterion_benchmark(c: &mut Criterion) {
    // cargo run --release graph ~/cloudflare_sync/severance_pbfs/elephant_castle.pbf ~/Downloads/uk_gtfs/
    let bytes = std::fs::read("graph.bin").unwrap();
    let map = MapModel::from_graph_bytes(&bytes).unwrap();

    // Fixed seed for determinism
    let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(42);
    let bounds = map.get_bounds();

    let mut group = c.benchmark_group("calculate routes");
    // TODO Try just a few inputs, because criterion repeats each input so many times
    for case in 0..3 {
        let (x1, y1) = rand_pt(&mut rng, &bounds);
        let (x2, y2) = rand_pt(&mut rng, &bounds);
        let req = RouteRequest {
            x1,
            y1,
            x2,
            y2,
            mode: "transit".to_string(),
            debug_search: false,
            use_heuristic: true,
            start_time: "07:00".to_string(),
        };

        group.bench_with_input(
            BenchmarkId::new("calculate routes", case),
            &req,
            |b, input| b.iter(|| map.route_from_req(input)),
        );
    }
    group.finish();
}

// TODO Or get the boundary polygon and ensure it's inside there
fn rand_pt(rng: &mut XorShiftRng, bounds: &Vec<f64>) -> (f64, f64) {
    (
        rng.gen_range(bounds[0]..bounds[2]),
        rng.gen_range(bounds[1]..bounds[3]),
    )
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
