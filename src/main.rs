use clap::Parser;
use std::io::Write;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The name of the .gpx file to process
    filename: String,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, EnumIter, num_enum::TryFromPrimitive)]
enum Waypoint {
    Generic = 0,
    Checkpoint,
    Water,
    Coffee,
    Grocery,
    RestArea,
    Accomodation,
    Restaurant,
    Warning,
    Camping,
    Ferry,
    Summit,
    Valley,
    Viewpoint,
}

fn main() {
    let args = Args::parse();
    let file = std::fs::File::open(&args.filename).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut gpx: gpx::Gpx = gpx::read(reader).unwrap();

    for waypoint in &mut gpx.waypoints {
        for wp in Waypoint::iter() {
            println!("{}) {:?}", wp as u8, wp);
        }
        print!(
            "Select type of waypoint '{}' of options above: ",
            waypoint.name.as_ref().unwrap()
        );
        std::io::stdout().flush().unwrap();

        let mut selected_type_input = String::from("");
        std::io::stdin()
            .read_line(&mut selected_type_input)
            .expect("failed to readline");

        let selected_type =
            Waypoint::try_from(selected_type_input.trim().parse::<u8>().unwrap()).unwrap();

        let waypoint_type = match selected_type {
            Waypoint::Generic => "generic",
            Waypoint::Checkpoint => "checkpoint",
            Waypoint::Water => "water",
            Waypoint::Coffee => "coffee",
            Waypoint::Grocery => "grocery",
            Waypoint::RestArea => "rest_area",
            Waypoint::Accomodation => "lodging",
            Waypoint::Restaurant => "food",
            Waypoint::Warning => "warning",
            Waypoint::Camping => "campsite",
            Waypoint::Ferry => "ferry",
            Waypoint::Summit => "summit",
            Waypoint::Valley => "valley",
            Waypoint::Viewpoint => "viewpoint",
        };
        waypoint.type_ = Some(waypoint_type.to_owned());
        waypoint.symbol = Some(waypoint_type.to_owned());
    }

    let out_file = std::fs::File::create(format!(
        "{}_fixed.gpx",
        args.filename.strip_suffix(".gpx").unwrap()
    ))
    .unwrap();
    let writer = std::io::BufWriter::new(out_file);
    gpx::write(&gpx, writer).unwrap();
}
