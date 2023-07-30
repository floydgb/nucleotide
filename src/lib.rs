pub mod new;
pub mod prev;

#[cfg(test)]
mod test {
    use std::time;

    use super::*;

    #[test]
    fn test() {
        // Compare the times of the two runs
        println!("\nRunning prev ----------------------------");
        let now = time::Instant::now();
        prev::run();
        let prev_time = now.elapsed();
        println!("\nRunning new ----------------------------");
        let now = time::Instant::now();
        new::run();
        let new_time = now.elapsed();
        println!("\nprev: {:.2?}, new: {:.2?}\n", prev_time, new_time);
    }
}
