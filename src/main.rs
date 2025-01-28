use kiesraad_model::*;

#[cfg(not(feature = "validate"))]
fn main() {
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

#[cfg(feature = "validate")]
fn main() {
    if std::env::args().len() <= 1 {
        eprintln!("usage: validate <files>");
        return;
    }

    for data_source in std::env::args().skip(1) {
        let data_source = std::path::Path::new(&data_source);
        let records = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_path(data_source)
            .unwrap()
            .records()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let ignored = [
            "AantalBlancoStemmen",
            "AantalGeldigeStemmen",
            "AantalOngeldigeStemmen",
            "Kiesgerechtigden",
            "Opkomst",
        ];

        let records = records.chunk_by(|x, y| x.get(1) == y.get(1)).map(|record| {
            (
                record[0].get(0).unwrap(),
                record
                    .iter()
                    .filter_map(|x| {
                        (!ignored.contains(&x.get(2).unwrap())).then_some(Votes(
                            x.get(4).unwrap().parse::<Count>().unwrap_or_default(),
                        ))
                    })
                    .collect::<Vec<_>>(),
                record
                    .iter()
                    .filter_map(|x| {
                        (!ignored.contains(&x.get(2).unwrap())).then_some(Seats::filled(
                            x.get(5).unwrap().parse::<Count>().unwrap_or_default(),
                        ))
                    })
                    .collect::<Vec<_>>(),
            )
        });

        for record in records {
            let (id, votes, mut outcome) = record;
            if data_source.file_name().unwrap() == "uitslag_GR20220316_Gemeente.csv"
                && id == "Enkhuizen"
            {
                // this party only had one candidate in 2022
                outcome[3].limit = 1;
            } else if data_source.file_name().unwrap() == "uitslag_TK19480707_Nederland.csv" {
                // there is a "Overig" party here
                outcome[7].limit = 0;
            }

            #[cfg(feature = "rand-validate")]
            let (votes, outcome): (Vec<_>, Vec<_>) = {
                use rand::seq::SliceRandom;
                let mut mingle = std::iter::zip(votes, outcome).collect::<Vec<_>>();
                mingle.shuffle(&mut rand::rng());
                mingle.into_iter().unzip()
            };

            let total_seats = outcome.iter().map(|x| x.count()).sum();
            println!("checking {}:{id}", data_source.display());

            let mut seats = outcome
                .iter()
                .map(|x| Seats::limited(x.limit))
                .collect::<Vec<_>>();

            let file_name = data_source.file_name().unwrap().to_string_lossy();
            if file_name.starts_with("uitslag_TK") || file_name.starts_with("uitslag_EP") {
                allocate_national(Seats::filled(total_seats), votes, &mut seats);
            } else {
                allocate(Seats::filled(total_seats), votes, &mut seats);
            }

            assert_eq!(
                seats.iter().map(|x| x.count()).collect::<Vec<_>>(),
                outcome.iter().map(|x| x.count()).collect::<Vec<_>>()
            );
        }
    }
}
