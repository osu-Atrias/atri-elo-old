use crate::solve_itp;

#[test]
fn solve_itp_tests() {
    fn f(x: f64) -> f64 {
        x.powi(3) - x - 2.0
    }

    assert!(f(dbg!(solve_itp((1.0, 2.0), f))) < 1e-10);
}
