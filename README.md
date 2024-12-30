# RTE TEMPO BOT RS

A bot that sends a message over Telegram and/or MQTT when RTE TEMPO days are published online.
Written in Rust 🦀.

Licence : MIT

## Usage

```
Usage: tempo_bot_rs_2024 [OPTIONS]

Options:
  -e, --enable-mqtt
  -e, --enable-telegram
  -l, --loop-time-min <LOOP_TIME_MIN>          [default: 120]
  -p, --poll-history-file <POLL_HISTORY_FILE>  [default: poll_history.txt]
  -h, --help                                   Print help
  -V, --version                                Print version
```

You need to populate a file named `.env` with the following variables : 

```
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
MQTT_IP=
MQTT_PORT=
MQTT_USERNAME=
MQTT_PASSWORD=
MQTT_TOPIC=
MQTT_IDENTITY=
```

## Message format

Telegram :
```
Aujourd'hui lundi 30 décembre 2024 est rouge ♨️. Tarif maximal.
Demain mardi 31 décembre 2024 sera blanc 🏳️. Tarif intermédiaire.
```

MQTT : 
```json
{"day" : "2024-12-31" , "state" : "BLANC"}
```
