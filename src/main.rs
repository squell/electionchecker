use kiesraad_model::*;

macro_rules! votes {
    ($($x: expr),* $(,)?) => {
        vec![$(Votes($x),)*]
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
    fn run_election(target: Count, votes: Vec<Votes>) {
        println!(
            "running an election for {target} seats, parties: {votes:?}, using largest {}",
            if target >= 19 {
                "averages"
            } else {
                "surpluses"
            }
        );
        let mut seats = vec![Seats::unlimited(); votes.len()];
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

    fn run_national_election(votes: Vec<Votes>) {
        println!("running an election for Tweede Kamer");
        let mut seats = vec![Seats::unlimited(); votes.len()];
        allocate_national(Seats::filled(150), votes, &mut seats);
        print_seats(seats.into_iter());
        println!("======");
    }

    #[rustfmt::skip]
    run_national_election(votes![
        2_450_878,
        1_643_073,
        1_589_519,
        1_343_287,
          656_292,
          485_551,
          345_822,
          328_225,
          246_765,
          235_148,
          232_963,
          217_270,
          212_532,
          178_802,
           71_345,
           52_913,
           51_043,
           44_253,
           12_838,
            9_117,
            5_487,
            5_325,
            5_122,
            4_152,
            3_966,
            1_038,
    ]);
}
