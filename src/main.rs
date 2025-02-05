#[cfg(feature = "validate")]
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use kiesraad_model::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run some example elections
    Demo,
    /// Run an election with the provided number of seats and votes
    Allocate(AllocateArgs),
    /// Validate election results from CSV file(s)
    #[cfg(feature = "validate")]
    Validate { files: Vec<PathBuf> },
}

#[derive(Args)]
struct AllocateArgs {
    /// Total number of seats to allocate
    seats: u64,
    /// Number of votes per party
    #[clap(num_args = 1.., value_delimiter=',')]
    votes: Vec<u64>,
    /// Use a voting threshold of one whole seat, as used in Dutch national elections
    #[arg(short, long)]
    national: bool,
}

fn main() {
    println!(
        "Copyright (C) 2025  Marc Schoolderman
This program comes with ABSOLUTELY NO WARRANTY
This is free software, and you are welcome to redistribute it
under certain conditions, see the file LICENSE
"
    );

    let cli = Cli::parse();

    match &cli.command {
        Command::Demo => demo(),
        Command::Allocate(args) => {
            let votes = args.votes.iter().map(|v| Votes(*v)).collect::<Vec<Votes>>();
            println!(
                "running an election for {} seats, parties: {votes:?}, using largest {}",
                args.seats,
                if args.national {
                    "averages (with voting threshold of one whole seat)"
                } else if args.seats >= 19 {
                    "averages"
                } else {
                    "surpluses"
                }
            );
            let mut seats = vec![Seats::unlimited(); votes.len()];
            if args.national {
                allocate_national(Seats::filled(args.seats), &votes, &mut seats);
            } else {
                allocate(Seats::filled(args.seats), &votes, &mut seats);
            }
            print_seats(seats.into_iter());
        }
        #[cfg(feature = "validate")]
        Command::Validate { files } => {
            println!("Validating {} files...", files.len());
            validate(files);
        }
    }
}

fn print_seats(seats: impl Iterator<Item = Seats>) {
    print!("result = ");
    for seat in seats {
        print!("{seat}, ");
    }
    println!();
}

fn demo() {
    macro_rules! votes {
    ($($x: expr),* $(,)?) => {
        vec![$(Votes($x),)*]
    }
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
        allocate_per_surplus(Seats::filled(target), &votes, &mut seats);
        print_seats(seats.into_iter());
        println!("======");
    }

    run_election(65432, votes![65535, 10]);
    run_election(65432, votes![65536, 10]);
}

#[cfg(feature = "validate")]
fn validate(data_sources: &Vec<PathBuf>) {
    for data_source in data_sources {
        let records = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_path(data_source)
            .unwrap()
            .records()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let ignored = |s: &str| {
            [
                "AantalBlancoStemmen",
                "AantalGeldigeStemmen",
                "AantalOngeldigeStemmen",
                "Kiesgerechtigden",
                "Opkomst",
            ]
            .contains(&s)
        };

        let records = records.chunk_by(|x, y| x[1] == y[1]).map(|record| {
            (
                &record[0][0],
                record
                    .iter()
                    .filter_map(|x| {
                        (!ignored(&x[2])).then_some(Votes(x[4].parse().unwrap_or_default()))
                    })
                    .collect::<Vec<_>>(),
                record
                    .iter()
                    .filter_map(|x| {
                        (!ignored(&x[2])).then_some(Seats::filled(x[5].parse().unwrap_or_default()))
                    })
                    .collect::<Vec<_>>(),
                record
                    .iter()
                    .filter_map(|x| {
                        (!ignored(&x[2])).then_some(
                            x[6].parse()
                                .map(Seats::limited)
                                .unwrap_or(Seats::unlimited()),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        });

        for (id, ref votes, outcome, candidates) in records {
            #[cfg(feature = "rand-validate")]
            let (ref votes, outcome): (Vec<_>, Vec<_>) = {
                use rand::seq::SliceRandom;
                let mut mingle = std::iter::zip(votes, outcome).collect::<Vec<_>>();
                mingle.shuffle(&mut rand::rng());
                mingle.into_iter().unzip()
            };

            let total_seats = outcome.iter().map(|x| x.count()).sum();
            println!("checking {}:{id}", data_source.display());

            let mut seats = candidates;

            let file_name = data_source.file_name().unwrap().to_string_lossy();
            if file_name.starts_with("uitslag_TK") || file_name.starts_with("uitslag_EP") {
                match &file_name[10..14] {
                    "1918" => allocate_1918(Seats::filled(total_seats), votes, &mut seats),
                    "1922" => allocate_1922(Seats::filled(total_seats), votes, &mut seats),
                    "1925" | "1929" | "1933" => {
                        allocate_bongaerts(Seats::filled(total_seats), votes, &mut seats)
                    }
                    _ => allocate_national(Seats::filled(total_seats), votes, &mut seats),
                }
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
