use kiesraad_model::*;

macro_rules! votes {
    ($($x: expr),* $(,)?) => {
        [$(Votes($x),)*]
    }
}

fn print_seats(seats: impl Iterator<Item = Seats>) {
    print!("result = ");
    for seat in seats {
        print!("{seat}, ");
    }
    println!();
}

fn main() {
    fn run_election<const N: usize>(target: Count, votes: [Votes; N]) {
        println!(
            "running an election for {target} seats, parties: {votes:?}, using largest {}",
            if target >= 19 {
                "averages"
            } else {
                "surpluses"
            }
        );
        let mut seats = std::array::from_fn(|_| Seats::unlimited());
        allocate(Seats::filled(target), votes, &mut seats);
        print_seats(seats.into_iter());
        println!("======");
    }

    run_election(19, votes![40, 30, 20, 10]);
    run_election(24, votes![21, 20]);
    run_election(20, votes![51, 25, 25]);
    run_election(50, votes![26, 25]);

    run_election(5, votes![19, 19, 19, 19, 15, 9, 9]);
    run_election(18, votes![100, 16, 6, 5, 5, 5, 5, 4]);
}
