use std::time::Instant;


/// Measures the latency of a function call in milliseconds.
/// /// # Arguments
/// /// * `f` - A closure that takes no arguments and returns a value of type `T`.
/// /// # Returns
/// /// A tuple containing the result of the function call and the elapsed time in milliseconds.

pub fn measure_latency<F, T>(f: F) -> (T, f32)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed().as_secs_f32() * 1000.0; // Convert to ms
    (result, duration)
}

