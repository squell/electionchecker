mod data;

pub use data::*;
use std::iter;

/// This performs one step in an apportionment algorithm, allocating seats based on a
/// "criterion" for how 'worthy' a certain party in the `seats` list is to receive the seats.
/// It is a **requirement** that the `criterion` algorithm will always rank a party that is
/// eligible for at least one more "eat" above a party that doesn't.
/// The `criterion` can signal that a party isn't eligible for seats by returning `None`.
pub fn allocate_single_step<Quality: Ord>(
    votes: &[Votes],
    seats: &mut [Seats],
    available_seats: &mut Seats,
    criterion: impl Fn(Votes, Seats) -> Option<Quality>,
) -> Option<()> {
    let qualities = iter::zip(votes, seats.iter())
        .map(|(votes, seats)| {
            if seats.has_candidates() {
                criterion(*votes, *seats)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let max_quality = qualities.iter().max().unwrap().as_ref()?;

    let awarded = iter::zip(qualities.iter(), seats)
        .filter_map(|(quality, seat)| (quality.as_ref() == Some(max_quality)).then_some(seat))
        .collect::<Vec<_>>();

    for seat in balloted(awarded, available_seats.count()) {
        seat.transfer(available_seats);
    }

    Some(())
}

/// This performs the correction stipulated in the Dutch law that a party that gets an
/// absolute majority in votes also gets an absolute majority in seats.
/// It is **required** that `prev_seats` contains the seat allocation of the penultimate
/// step in the seat allocation algorithm (since that will determine who loses a seat).
/// This step is criterion-agnostic.
pub fn absolute_majority_check(votes: &[Votes], seats: &mut [Seats], prev_seats: Vec<Seats>) {
    let total_votes = votes.iter().map(|Votes(count)| count).sum::<Count>();
    let total_seats = seats.iter().map(|count| count.count()).sum::<Count>();

    let mut correction = Seats::filled(1);

    let absolute_majority = |count, total| 2 * count > total;

    if let Some((_, winner_seat)) =
        iter::zip(votes, seats.iter_mut()).find(|(Votes(cur_vote), cur_seat)| {
            cur_seat.has_candidates()
                && absolute_majority(*cur_vote, total_votes)
                && !absolute_majority(cur_seat.count(), total_seats)
        })
    {
        #[cfg(feature = "chatty")]
        eprintln!("an absolute majority correction was performed");
        winner_seat.transfer(&mut correction);
        let winner_seat = *winner_seat;

        let last_winners = iter::zip(seats.iter_mut(), prev_seats)
            .filter_map(|(x, y)| (*x > y && *x != winner_seat).then_some(x))
            .collect::<Vec<_>>();

        let loser_seat = balloted(last_winners, 1).next().unwrap();
        correction.transfer(loser_seat);
    }
}

#[cfg(feature = "chatty")]
pub fn whole_seats_available(votes: &[Votes], seats: &[Seats], seats_awarded: Seats) -> bool {
    let total_seats = seats_awarded.count() + seats.iter().map(|x| x.count()).sum::<Count>();
    let total_votes = votes.iter().map(|Votes(x)| x).sum::<Count>();
    iter::zip(votes, seats).any(|(Votes(cur_vote), cur_seat)| {
        frac(*cur_vote, cur_seat.count() + 1) >= frac(total_votes, total_seats)
    })
}

#[cfg(feature = "chatty")]
fn debug_results(mut things: impl Iterator<Item: std::fmt::Display>) {
    let Some(first) = things.next() else {
        return;
    };
    eprint!("{first}");
    for thing in things {
        eprint!(", {thing}");
    }
    eprintln!();
}

/// Perform a seat apportionment based on the given method.
/// It is a **requirement** that the `criterion` algorithm will always rank a party that is
/// eligible for at least one more "eat" above a party that doesn't.
pub fn allocate_seats<Quality: Ord>(
    votes: &[Votes],
    seats: &mut [Seats],
    available_seats: &mut Seats,
    method: impl Fn(Votes, Seats) -> Option<Quality> + Copy,
) {
    let mut last_winners = seats.to_owned();
    #[cfg(feature = "chatty")]
    let mut printed = false;
    while available_seats.count() > 0 {
        #[cfg(feature = "chatty")]
        if !(whole_seats_available(votes, seats, *available_seats) || printed) {
            printed = true;
            eprintln!("whole seats:");
            debug_results(
                seats
                    .iter()
                    .enumerate()
                    .filter_map(|(n, x)| (x.count() > 0).then_some(format!("{n}: {x}"))),
            );
            eprintln!("rest seats ({})", available_seats.count());
        }

        last_winners.copy_from_slice(seats);

        if allocate_single_step(votes, seats, available_seats, method).is_none() {
            return;
        }

        #[cfg(feature = "chatty")]
        if cfg!(feature = "succinct-chatty") {
            if printed {
                debug_results(
                    seats
                        .iter()
                        .zip(last_winners.iter())
                        .enumerate()
                        .filter_map(|(n, (x, y))| (x != y).then_some(format!("rest seat for {n}"))),
                );
            }
        } else {
            debug_results(seats.iter());
        }
    }

    absolute_majority_check(votes, seats, last_winners);
}

/// Perform a seat apportionment, only handing out full seats. This is not necessary but has the
/// benefit that it is criterion-agnostic and faster than an explicit loopp.
#[allow(unused)]
pub fn allocate_whole_seats(votes: &[Votes], seats: &mut [Seats], available_seats: &mut Seats) {
    let vote_count = votes.iter().map(|Votes(count)| count).sum::<Count>();
    let seat_count = available_seats.count();

    for (Votes(v), seat) in iter::zip(votes.iter(), seats.iter_mut()) {
        for _ in 0..v * seat_count / vote_count {
            if seat.count() < seat.limit {
                seat.transfer(available_seats)
            }
        }
    }
}

/// Perform a seat apportionment based on the D'Hondt method.
/// This system is currently used in the Netherlands for regional councils least 19 seats or more.
pub fn allocate_per_average(mut total_seats: Seats, votes: &[Votes], seats: &mut [Seats]) {
    #[cfg(feature = "whole-seat-opt")]
    allocate_whole_seats(votes, seats, &mut total_seats);

    allocate_seats(
        votes,
        seats,
        &mut total_seats,
        |Votes(cur_vote), cur_seat| Some(frac(cur_vote, cur_seat.count() + 1)),
    );
}

/// Perform a seat apportionment based on the Hamilton method, with a
/// voting threshold of 75% of a whole seat, and parties receiving a maximum of one extra seat.
/// If seats remain after that, apportion the remainder of seats using D'Hondt, with
/// parties again only receiving a maximum of one additional seat.
/// This system is currently used in the Netherlands for bodies of less than 19 seats.
pub fn allocate_per_surplus(mut total_seats: Seats, votes: &[Votes], seats: &mut [Seats]) {
    let vote_count = votes.iter().map(|Votes(count)| count).sum::<Count>();
    let seat_count = total_seats.count();

    let has_surplus =
        |cur_vote, cur_seat| frac(cur_vote, 1) >= frac(cur_seat * vote_count, seat_count);

    #[cfg(feature = "whole-seat-opt")]
    allocate_whole_seats(votes, seats, &mut total_seats);

    allocate_seats(
        votes,
        seats,
        &mut total_seats,
        move |Votes(cur_vote), cur_seat| {
            let cur_seat = cur_seat.count();
            (has_surplus(cur_vote, cur_seat)
                && frac(cur_vote, 1) >= frac(3 * vote_count, 4 * seat_count))
            //            .then(|| cur_vote * seat_count - cur_seat * vote_count)
            .then(|| cur_vote - cur_seat * vote_count / seat_count)
        },
    );

    if total_seats.count() > 0 {
        #[cfg(feature = "chatty")]
        eprintln!("continuing by averages");
        allocate_seats(
            votes,
            seats,
            &mut total_seats,
            |Votes(cur_vote), cur_seat| {
                let cur_seat = cur_seat.count();
                if frac(cur_vote, 1) >= frac(3 * vote_count, 4 * seat_count) {
                    has_surplus(cur_vote, cur_seat - 1)
                } else {
                    has_surplus(cur_vote, cur_seat)
                }
                .then_some(frac(cur_vote, cur_seat + 1))
            },
        );
    }
}

/// Perform a seat apportionment, selecting D'Hondt or modified-Hamilton
/// based on the number of seats, as Dutch law does for bodies.
pub fn allocate(total_seats: Seats, votes: &[Votes], seats: &mut [Seats]) {
    if total_seats.count() >= 19 {
        allocate_per_average(total_seats, votes, seats);
    } else {
        allocate_per_surplus(total_seats, votes, seats);
    }
}

/// Perform a seat apportionment using D'Hondt's method and a voting threshold
/// of one whole seat, as used in Dutch national elections (parliament and European Parliament)
pub fn allocate_national(mut total_seats: Seats, votes: &[Votes], seats: &mut [Seats]) {
    let vote_count = votes.iter().map(|Votes(count)| count).sum::<Count>();
    let seat_count = total_seats.count();

    #[cfg(feature = "whole-seat-opt")]
    allocate_whole_seats(votes, seats, &mut total_seats);

    allocate_seats(
        votes,
        seats,
        &mut total_seats,
        |Votes(cur_vote), cur_seat| {
            (frac(cur_vote, 1) >= frac(vote_count, seat_count))
                .then_some(frac(cur_vote, cur_seat.count() + 1))
        },
    );
}

/// Perform a seat apportionment using the method that seems to have been in place from 1925 until
/// the introduction of D'Hondt method, at least for the national election. It is the single-seat
/// Hamilton method. And an extra requirement that a party always needs to have 75% of a whole seat
/// *on average*, which acts like a quite ingenious voting threshold.
/// If seats remain, they are then apportioned by the "single-additional seat D'Hondt" method.
pub fn allocate_bongaerts(mut total_seats: Seats, votes: &[Votes], seats: &mut [Seats]) {
    let vote_count = votes.iter().map(|Votes(count)| count).sum::<Count>();
    let seat_count = total_seats.count();

    let has_surplus =
        |cur_vote, cur_seat| frac(cur_vote, 1) >= frac(cur_seat * vote_count, seat_count);

    #[cfg(feature = "whole-seat-opt")]
    allocate_whole_seats(votes, seats, &mut total_seats);

    allocate_seats(
        votes,
        seats,
        &mut total_seats,
        move |Votes(cur_vote), cur_seat| {
            let cur_seat = cur_seat.count();
            // proposed by bongaerts in 1922 and adopted in law in 1925
            (has_surplus(cur_vote, cur_seat)
                && frac(cur_vote, cur_seat + 1) >= frac(3 * vote_count, 4 * seat_count))
            .then(|| cur_vote * seat_count - cur_seat * vote_count)
        },
    );

    if total_seats.count() > 0 {
        #[cfg(feature = "chatty")]
        eprintln!("continuing by averages");
        allocate_seats(
            votes,
            seats,
            &mut total_seats,
            //bongaerts in 1922 seems to have proposed this instead (straight saint-laguë method)
            //|Votes(cur_vote), cur_seat| Some(frac(2*cur_vote, 2*cur_seat.count() + 1)),
            |Votes(cur_vote), cur_seat| {
                let cur_seat = cur_seat.count();
                if cur_seat > 0 && frac(cur_vote, cur_seat) >= frac(3 * vote_count, 4 * seat_count)
                {
                    has_surplus(cur_vote, cur_seat - 1)
                } else {
                    has_surplus(cur_vote, cur_seat)
                }
                .then_some(frac(cur_vote, cur_seat + 1))
            },
        );
    }
}

/// The seat apportionment used in the very first election with proportional representation.
pub fn allocate_1918(total_seats: Seats, votes: &[Votes], seats: &mut [Seats]) {
    allocate_archaic(frac(1, 2), total_seats, votes, seats);
}

/// The seat apportionment used in the strange 1922 election.
/// This has an increased voting threshold of 75% instead of the original 50%.
pub fn allocate_1922(total_seats: Seats, votes: &[Votes], seats: &mut [Seats]) {
    allocate_archaic(frac(3, 4), total_seats, votes, seats);
}

/// Perform a seat apportionment using the method that seems to have been selected around 1916
/// For the big introduction of proportional representation. It is essentially the Hamilton method
/// with a voting threshold (for two rounds, after it will weirdly only take into consideration
/// parties that *do not meet* the voting threshold). This gives a tremendous boost for small parties
/// as rest seats can are apportioned rather randomly after round 1.
///
/// The second and third rounds don't seem to have ever been needed---a second round would have
/// been needed in 1922, except that two major parties prevented this by hacking the electoral
/// system and artificially making themselves smaller by splitting into multiple lists.
///
/// This silly system was abandoned in 1922.
pub fn allocate_archaic(
    mut threshold: Fraction,
    mut total_seats: Seats,
    votes: &[Votes],
    seats: &mut [Seats],
) {
    let vote_count = votes.iter().map(|Votes(count)| count).sum::<Count>();
    let seat_count = total_seats.count();

    threshold.numerator *= vote_count;
    threshold.denominator *= seat_count;

    let has_surplus =
        |cur_vote, cur_seat| frac(cur_vote, 1) >= frac(cur_seat * vote_count, seat_count);

    #[cfg(feature = "whole-seat-opt")]
    allocate_whole_seats(votes, seats, &mut total_seats);

    let mut round = |num, meet_threshold| {
        if total_seats.count() > 0 {
            #[cfg(feature = "chatty")]
            if num > 0 {
                eprintln!("entering second round of surplus apportionment");
            }
            allocate_seats(
                votes,
                seats,
                &mut total_seats,
                move |Votes(cur_vote), cur_seat| {
                    (((frac(cur_vote, 1) >= threshold) == meet_threshold)
                        && has_surplus(cur_vote, cur_seat.count() - num))
                    .then(|| cur_vote * seat_count - (cur_seat.count() - num) * vote_count)
                },
            );
        }
    };

    // this is my best interpretation from a 1917 law
    round(0, true);
    round(1, true);
    round(0, false);
}
