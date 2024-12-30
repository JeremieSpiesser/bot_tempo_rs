use std::{env, time::Duration};

use clap::{arg, command, Parser};
use dotenv::dotenv;
use log::{info, warn};
use mqtt::MQTTHandler;
use state::State;
use telegram::TGBot;
use tempo::EdfTempoStore;
use tokio::time::sleep;

mod mqtt;
mod state;
mod telegram;
mod tempo;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = true)]
    enable_mqtt: bool,

    #[arg(short, long, default_value_t = true)]
    enable_telegram: bool,

    #[arg(short, long, default_value_t = 120)]
    loop_time_min: u64,

    #[arg(short, long, default_value = "poll_history.txt")]
    poll_history_file: String,
}

#[tokio::main]
async fn main() {
    info!("Starting tempo bot rs...");
    env_logger::init();
    dotenv().ok();
    let args = Args::parse();

    let handled_day_store = State::new(&args.poll_history_file);

    let tgbot: Option<TGBot> = if args.enable_telegram {
        let tg_token = env::var("TELEGRAM_BOT_TOKEN")
            .expect("Please specify your telegram bot token with TELEGRAM_BOT_TOKEN");
        let tg_chat_id = env::var("TELEGRAM_CHAT_ID")
            .expect("Please specify your telegram chat id with TELEGRAM_CHAT_ID");
        Some(TGBot::new(&tg_token, &tg_chat_id))
    } else {
        None
    };

    let mqtt: Option<MQTTHandler> = if args.enable_mqtt {
        let ip =
            &env::var("MQTT_IP").expect("Please specify your MQTT server IP address with MQTT_IP");
        let port = env::var("MQTT_PORT")
            .expect("Please specify your MQTT server PORT with MQTT_PORT")
            .parse::<u16>()
            .unwrap_or_else(|_| {
                warn!("Could not convert mqtt port. Defaulting to port 1883");
                1883
            });
        let username = env::var("MQTT_USERNAME")
            .expect("Please specify your MQTT username with MQTT_USERNAME");
        let password = env::var("MQTT_PASSWORD")
            .expect("Please specify your MQTT password with MQTT_PASSWORD");
        let topic = env::var("MQTT_TOPIC").expect("Please specify your MQTT topic with MQTT_TOPIC");
        let identity = if let Ok(identity) = env::var("MQTT_IDENTITY") {
            Some(identity)
        } else {
            None
        };

        Some(MQTTHandler::new(
            ip,
            port,
            username.as_str(),
            password.as_str(),
            topic.as_str(),
            identity.as_deref(),
        ))
    } else {
        None
    };

    info!("Initialization done. Now looping...");

    loop {
        sleep(Duration::from_secs(60 * args.loop_time_min)).await;
        info!("Loop wait done. Analyzing situation.");
        let now = chrono::Utc::now();
        let today_string = now.format("%Y-%m-%d").to_string();
        info!("Today : {}", today_string);
        let tomorrow = now.checked_add_days(chrono::Days::new(1)).unwrap();
        let tomorrow_string = tomorrow.format("%Y-%m-%d").to_string();
        info!("Tomorrow : {}", tomorrow_string);

        let last_handled_day = handled_day_store.get().unwrap();
        info!("Last polled day is {}", last_handled_day);

        if last_handled_day == tomorrow_string {
            info!("Skipping...");
            continue;
        }

        info!("Entering a new day. Polling RTE");

        let edfstore = EdfTempoStore::retrieve_info().await.unwrap();
        let tomorrow_tempo_day = edfstore.retrieve_tempo_day_state(tomorrow_string.as_str());
        let today_tempo_day = edfstore.retrieve_tempo_day_state(today_string.as_str());

        if let Some(tomorrow_tempo_day) = tomorrow_tempo_day {
            info!(
                "Received new info about tomorrow {}",
                tomorrow_tempo_day.day
            );
            if let Some(ref tgbot) = tgbot {
                info!("Sending telegram message");
                tgbot
                    .handle_day(today_tempo_day, Some(tomorrow_tempo_day.clone()))
                    .await;
                info!("Done");
            }

            if let Some(ref mqtt) = mqtt {
                info!("Sending MQTT event");
                mqtt.handle_day(&tomorrow_tempo_day).await;
                info!("Done");
            }
            let _ = handled_day_store.set(&tomorrow_tempo_day.day);
        }
    }
}
