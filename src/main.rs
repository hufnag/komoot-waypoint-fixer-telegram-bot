use geo::{Distance, InterpolatableLine, LineLocatePoint, coord};
use std::{
    ops::DerefMut,
    sync::{Arc, OnceLock},
};
use strum::IntoEnumIterator;
use teloxide::{
    dispatching::dialogue::GetChatId,
    net::Download,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MediaKind, MessageKind},
};
use tokio::sync::Mutex;
mod waypoint;
use waypoint::Waypoint;
mod app_state;
use app_state::AppState;

type SharedAppState = Arc<Mutex<AppState>>;

static TMP_FILE_DIR: OnceLock<String> = OnceLock::new();

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    TMP_FILE_DIR.get_or_init(|| std::env::var("WAYPOINT_FIXER_TMP_DIR").unwrap_or(".".to_string()));

    let bot = Bot::from_env();

    let app_state = Arc::new(Mutex::new(AppState::default()));
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    log::info!("Starting dispatcher...");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn handle_message(bot: Bot, msg: Message, app_state: SharedAppState) -> ResponseResult<()> {
    if let MessageKind::Common(ref common_msg) = msg.kind
        && let MediaKind::Document(ref document) = common_msg.media_kind
    {
        let file = bot.get_file(document.document.file.id.clone()).await?;

        let file_name = document
            .document
            .file_name
            .as_ref()
            .expect("Received document needs to have a filename set.");
        log::info!("Received file: {file_name}");
        if !file_name.ends_with(".gpx") {
            log::warn!("Received file is not a GPX file: {file_name}. Ignoring.");
            return Ok(());
        }
        let gpx_file_destination = format!("{}/{file_name}", TMP_FILE_DIR.get().unwrap());
        let mut gpx_file = tokio::fs::File::create(&gpx_file_destination)
            .await
            .unwrap();
        bot.download_file(&file.path, &mut gpx_file).await.unwrap();
        let file = std::fs::File::open(&gpx_file_destination).unwrap();
        let reader = std::io::BufReader::new(file);
        let gpx: gpx::Gpx = match gpx::read(reader) {
            Ok(gpx) => gpx,
            Err(e) => {
                log::error!("Failed to parse GPX file: {e}");
                return Ok(());
            }
        };

        if let Some(waypoint) = gpx.waypoints.first() {
            send_waypoint(waypoint, &bot, &msg.chat_id().unwrap()).await;
        } else {
            log::info!("No waypoints found in GPX file");
        }
        *app_state.lock().await.deref_mut() = AppState {
            gpx,
            waypoint_index: 0,
            gpx_file_name: file_name.to_string(),
        };
    }
    Ok(())
}

async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    app_state: SharedAppState,
) -> ResponseResult<()> {
    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;
        let mut state = app_state.lock().await;
        let index = state.waypoint_index;
        log::info!(
            "Setting type and symbol of waypoint '{}' to '{}'",
            state.gpx.waypoints[index]
                .name
                .clone()
                .expect("Waypoints need to have a name set."),
            data
        );

        fix_waypoint(&mut state.gpx, &data, index);

        if index + 1 == state.gpx.waypoints.len() {
            log::info!("All waypoints processed. Sending back fixed GPX file.");
            let fixed_gpx_file_name = format!(
                "{}/{}_fixed.gpx",
                TMP_FILE_DIR.get().unwrap(),
                state.gpx_file_name.strip_suffix(".gpx").unwrap()
            );
            let fixed_gpx_file = std::fs::File::create(&fixed_gpx_file_name).unwrap();
            let fixed_gpx_file_writer = std::io::BufWriter::new(fixed_gpx_file);
            gpx::write(&state.gpx, fixed_gpx_file_writer).unwrap();
            let chat_id = q.message.as_ref().unwrap().chat().chat_id().unwrap();
            bot.send_document(chat_id, InputFile::file(fixed_gpx_file_name))
                .await
                .unwrap();
        } else {
            state.waypoint_index += 1;
            send_waypoint(
                &state.gpx.waypoints[state.waypoint_index],
                &bot,
                &q.message.unwrap().chat().chat_id().unwrap(),
            )
            .await;
        }
    }

    Ok(())
}

async fn send_waypoint(waypoint: &gpx::Waypoint, bot: &Bot, chat_id: &ChatId) {
    let buttons = Waypoint::iter()
        .map(|wp| {
            InlineKeyboardButton::callback(
                format!("{} {}", wp, wp.symbol()),
                wp.wahoo_waypoint_name(),
            )
        })
        .collect::<Vec<InlineKeyboardButton>>();

    const BUTTONS_PER_ROW: usize = 2;
    let keyboard = InlineKeyboardMarkup::new(
        buttons
            .chunks(BUTTONS_PER_ROW)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<Vec<InlineKeyboardButton>>>(),
    );

    bot.send_message(
        *chat_id,
        format!(
            "Choose type of Waypoint '{}':",
            waypoint.name.as_ref().unwrap()
        ),
    )
    .reply_markup(keyboard)
    .await
    .unwrap();
}

fn fix_waypoint(gpx: &mut gpx::Gpx, waypoint_type: &str, waypoint_index: usize) {
    let waypoint = &mut gpx.waypoints[waypoint_index];
    // 1. Set correct type and symbol
    waypoint.type_ = Some(waypoint_type.to_string());
    waypoint.symbol = Some(waypoint_type.to_string());

    // 2. Insert missing waypoint into track if not already present

    // Check if waypoint coordinates are already part of the track
    let waypont_point = geo::Point::new(waypoint.point().x(), waypoint.point().y());
    let track = gpx
        .tracks
        .first_mut()
        .unwrap()
        .segments
        .first_mut()
        .unwrap();

    if track
        .points
        .iter()
        .find(|point| point.point() == waypoint.point())
        .is_some()
    {
        log::info!("Waypoint coordinate is already part of the track. No further steps needed.");
        return;
    }

    let line_string: geo::LineString = track
        .points
        .iter()
        .map(|point| coord! {x: point.point().x(), y: point.point().y()})
        .collect();

    match line_string.line_locate_point(&waypont_point) {
        Some(ratio) => {
            let waypoint_projection = line_string
                .point_at_ratio_from_start(&geo::Euclidean, ratio)
                .unwrap();

            let index_to_insert_additional_waypoints =
                get_waypoint_insert_index_of_track(&line_string, &waypoint.point());
            let projection_waypoint = gpx::Waypoint::new(geo_types::Point::new(
                waypoint_projection.x(),
                waypoint_projection.y(),
            ));
            track.points.splice(
                index_to_insert_additional_waypoints..index_to_insert_additional_waypoints,
                [
                    projection_waypoint.clone(),
                    waypoint.clone(),
                    projection_waypoint,
                ],
            );
        }
        None => {
            log::error!("Could not find closest point on track for waypoint");
        }
    }
}

/// Returns the index at which to insert a waypoint into a track, based on the two closest exisiting points in the track
fn get_waypoint_insert_index_of_track(track: &geo::LineString, waypoint: &geo::Point) -> usize {
    let index_of_closest_point_to_projection = track
        .points()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            geo::Euclidean
                .distance(a, waypoint)
                .total_cmp(&geo::Euclidean.distance(b, waypoint))
        })
        .map(|(idx, _)| idx)
        .unwrap();

    let distance_to_point_before = track
        .points()
        .collect::<Vec<geo::Point>>()
        .get(index_of_closest_point_to_projection - 1)
        .map(|p| geo::Euclidean.distance(p, waypoint))
        .unwrap_or(f64::INFINITY);
    let distance_to_point_after = track
        .points()
        .collect::<Vec<geo::Point>>()
        .get(index_of_closest_point_to_projection + 1)
        .map(|p| geo::Euclidean.distance(p, waypoint))
        .unwrap_or(f64::INFINITY);
    if distance_to_point_after > distance_to_point_before {
        index_of_closest_point_to_projection
    } else {
        index_of_closest_point_to_projection + 1
    }
}
