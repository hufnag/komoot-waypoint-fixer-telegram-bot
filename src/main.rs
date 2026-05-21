use std::{ops::DerefMut, sync::Arc};
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

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

const TMP_FILE_DIR: &str = "/tmp/komoot_waypoint_fixer_telegram_bot";

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
        let gpx_file_destination = format!("{TMP_FILE_DIR}/{file_name}");
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
        state.gpx.waypoints[index].type_ = Some(data.clone());
        state.gpx.waypoints[index].symbol = Some(data);

        if index + 1 == state.gpx.waypoints.len() {
            log::info!("All waypoints processed. Sending back fixed GPX file.");
            let fixed_gpx_file_name = format!(
                "{TMP_FILE_DIR}/{}_fixed.gpx",
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
        .map(|wp| InlineKeyboardButton::callback(wp.symbol(), wp.wahoo_waypoint_name()))
        .collect::<Vec<InlineKeyboardButton>>();

    const BUTTONS_PER_ROW: usize = 8;
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
