use burnhop_server::reliability::{Profile, run};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut seconds = 20.;
    let mut profile = Profile::named("baseline")?;
    let mut players = 8;
    let mut churn = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seconds" => {
                i += 1;
                seconds = args
                    .get(i)
                    .ok_or("seconds")?
                    .parse()
                    .map_err(|_| "seconds")?;
            }
            "--profile" => {
                i += 1;
                profile = Profile::named(args.get(i).ok_or("profile")?)?;
            }
            "--players" => {
                i += 1;
                players = args
                    .get(i)
                    .ok_or("players")?
                    .parse()
                    .map_err(|_| "players")?;
            }
            "--churn" => churn = true,
            _ => {
                return Err(
                    "--seconds N --profile baseline|50|100|150|stall --players 2..8 --churn".into(),
                );
            }
        }
        i += 1;
    }
    run(seconds, profile, players, churn)
}
