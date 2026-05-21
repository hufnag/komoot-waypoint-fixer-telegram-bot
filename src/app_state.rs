#[derive(Default)]
pub struct AppState {
    pub gpx: gpx::Gpx,
    pub waypoint_index: usize,
    pub gpx_file_name: String,
}
