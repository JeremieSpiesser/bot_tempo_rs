use chrono::{DateTime, Locale, Utc};
use log::{debug, info};
use serde_json::{Map, Value};
use std::{collections::HashMap, str::FromStr, time::Duration};
use tokio::time::timeout;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TempoDayState {
    Bleu,
    Blanc,
    Rouge,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TempoDay {
    pub day: String,
    chronotime: DateTime<Utc>,
    pub state: TempoDayState,
}

impl TempoDay {
    pub fn new(s: &str, state: TempoDayState) -> Self {
        Self {
            day: s.to_string(),
            chronotime: chrono::DateTime::from_str(format!("{} 00:00:00+00:00", s).as_str())
                .unwrap(),
            state,
        }
    }

    pub fn to_mqtt_string(&self) -> String {
        let state = match self.state {
            TempoDayState::Bleu => "BLEU",
            TempoDayState::Blanc => "BLANC",
            TempoDayState::Rouge => "ROUGE",
        };
        format!("{{'day' : '{}' , 'state' : '{}'}}", self.day, state)
    }

    pub fn to_french_date_string(&self) -> String {
        let locale = Locale::fr_FR;
        self.chronotime
            .format_localized("%A %d %B %Y", locale)
            .to_string()
    }

    pub fn to_french_complete_string(&self) -> String {
        let datestring = self.to_french_date_string();
        let (prefix, verb) = if self.chronotime.date_naive() == chrono::Utc::now().date_naive() {
            (String::from("Aujourd'hui"), String::from("est"))
        } else {
            (String::from("Demain"), String::from("sera"))
        };

        match self.state {
            TempoDayState::Blanc => {
                format!(
                    "{} {} {} blanc 🏳️. Tarif intermédiaire.",
                    prefix, datestring, verb
                )
            }
            TempoDayState::Bleu => {
                format!("{} {} {} bleu 🌊. Tarif minimal.", prefix, datestring, verb)
            }
            TempoDayState::Rouge => {
                format!(
                    "{} {} {} rouge ♨️. Tarif maximal.",
                    prefix, datestring, verb
                )
            }
        }
    }
}

impl TempoDayState {
    pub fn from(s: &str) -> Option<Self> {
        match s {
            "BLUE" => Some(TempoDayState::Bleu),
            "RED" => Some(TempoDayState::Rouge),
            "WHITE" => Some(TempoDayState::Blanc),
            _ => None,
        }
    }
}

pub struct EdfTempoStore {
    pub entries: HashMap<String, TempoDay>,
}

impl EdfTempoStore {
    pub async fn retrieve_info() -> anyhow::Result<Self> {
        let url =
            "https://www.services-rte.com/cms/open_data/v1/tempo?season=2024-2025".to_string();
        let client = reqwest::Client::builder()
            .user_agent("curl/7.54.1")
            .build()?;

        debug!("Going to GET : {url}");

        let resp = timeout(Duration::from_secs(5), client.get(url).send()).await??;
        info!("Call to RTE succeeded");

        let json: Value = resp.json().await?;
        let content = json.get("values").unwrap().as_object().unwrap();

        let m = EdfTempoStore::edf_values_to_tempo_types(&content);
        Ok(Self { entries: m })
    }

    pub fn retrieve_tempo_day_state(&self, day: &str) -> Option<TempoDay> {
        self.entries.get(day).cloned()
    }

    fn edf_values_to_tempo_types(values: &Map<String, Value>) -> HashMap<String, TempoDay> {
        let mut map = HashMap::new();

        for v in values {
            if let Some(state) = TempoDayState::from(v.1.as_str().unwrap()) {
                map.insert(v.0.into(), TempoDay::new(v.0, state));
            }
        }
        map
    }
}
