#[repr(u8)]
#[derive(Debug, Copy, Clone, strum_macros::EnumIter)]
pub enum Waypoint {
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

impl std::fmt::Display for Waypoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Waypoint::Generic => "Generic📍",
            Waypoint::Checkpoint => "Checkpoint 📋",
            Waypoint::Water => "Water 💧",
            Waypoint::Coffee => "Coffee ☕",
            Waypoint::Grocery => "Grocery 🛒",
            Waypoint::RestArea => "RestArea 🪑",
            Waypoint::Accomodation => "Accomodation 🏨",
            Waypoint::Restaurant => "Restaurant 🍽️",
            Waypoint::Warning => "Warning ⚠️",
            Waypoint::Camping => "Camping ⛺️",
            Waypoint::Ferry => "Ferry 🚢",
            Waypoint::Summit => "Summit 🏔️",
            Waypoint::Valley => "Valley 📉",
            Waypoint::Viewpoint => "Viewpoint 📷",
        };
        write!(f, "{s}")
    }
}

impl Waypoint {
    pub fn wahoo_waypoint_name(&self) -> &str {
        match self {
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
        }
    }
    pub fn symbol(&self) -> &str {
        match self {
            Waypoint::Generic => "📍",
            Waypoint::Checkpoint => "📋",
            Waypoint::Water => "💧",
            Waypoint::Coffee => "☕",
            Waypoint::Grocery => "🛒",
            Waypoint::RestArea => "🪑",
            Waypoint::Accomodation => "🏨",
            Waypoint::Restaurant => "🍽️",
            Waypoint::Warning => "⚠️",
            Waypoint::Camping => "⛺️",
            Waypoint::Ferry => "🚢",
            Waypoint::Summit => "🏔️",
            Waypoint::Valley => "📉",
            Waypoint::Viewpoint => "📷",
        }
    }
}
