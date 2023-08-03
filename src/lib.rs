#![feature(stmt_expr_attributes)]

// Exports --------------------------------------------------------------------
pub mod new;
pub mod prev;

// Macros ---------------------------------------------------------------------
#[macro_export]
macro_rules! str {
     ($($x:expr),*) =>
        (vec![$($x.into()),*].into_iter().rev().collect::<Vec<String>>());
}

// Tests ----------------------------------------------------------------------
#[cfg(test)]
mod test_nucleotide {

    use std::process::{Command, Stdio};

    #[test]
    fn test_prev_matches_new_stdout() {
        let Ok(cmd) = Command::new("./target/release/nucleotide")
            .stdout(Stdio::piped())
            .spawn()
        else {
            assert!(false);
            return;
        };
        let Ok(out) = cmd.wait_with_output() else {
            assert!(false);
            return;
        };
        match String::from_utf8(out.stdout) {
            Ok(outstr) => {
                let lines: Vec<&str> = outstr.split('\n').collect();
                assert_eq!(
                    lines[..lines.len() / 2],
                    lines[lines.len() / 2..lines.len() - 1]
                );
            }
            _ => assert!(false),
        }
    }
}
