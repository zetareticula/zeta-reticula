use std::time::Instant;

pub fn measure_latency<F, T>(f: F) -> (T, f32)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed().as_secs_f32() * 1000.0; // Convert to ms
    (result, duration)
}