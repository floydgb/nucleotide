pub mod new;
pub mod prev;

#[cfg(test)]
mod test {
    use std::time;

    use super::*;

    #[test]
    fn test() {
        // Compare the times of the two runs
        let now = time::Instant::now();
        prev::run();
        let prev_time = now.elapsed();
        new::run();
        let new_time = now.elapsed() - prev_time;
        println!("\nprev: {:.2?}, new: {:.2?}\n", prev_time, new_time);
    }
}
