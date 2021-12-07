use time::{macros::datetime, Duration};

fn main() {
    let s = datetime!(2021-12-07 14:00:00.000000000 +00:00:00);
    println!("{}", s);
    let time = Duration::seconds_f64(432012.179165143);
    println!("{}", s + time);
}
