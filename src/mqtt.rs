use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, PubAck, Publish, QoS};

use crate::tempo::TempoDay;

pub struct MQTTHandler {
    topic: String,
    mqttoptions: MqttOptions,
}

impl MQTTHandler {
    pub fn new(
        ip: &str,
        port: u16,
        username: &str,
        password: &str,
        topic: &str,
        identity: Option<&str>,
    ) -> Self {
        let mut mqttoptions =
            MqttOptions::new(identity.unwrap_or("bot_tempo_rs").to_owned(), ip, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_credentials(username, password);

        Self {
            topic: topic.to_owned(),
            mqttoptions,
        }
    }

    pub async fn send_msg(&self, msg: &str) {
        let (client, mut eventloop) = AsyncClient::new(self.mqttoptions.clone(), 10);
        let _ = client
            .subscribe(self.topic.as_str(), QoS::AtMostOnce)
            .await
            .unwrap();
        let _ = client
            .publish(self.topic.as_str(), QoS::AtLeastOnce, false, msg.as_bytes())
            .await;

        let mut is_published = false;
        loop {
            let notification = eventloop.poll().await.unwrap();
            println!("Received = {:?}", &notification);
            match notification {
                rumqttc::Event::Incoming(e) => match e {
                    rumqttc::Packet::Publish(Publish {
                        dup: _,
                        qos: _,
                        retain: _,
                        topic: _,
                        pkid: _,
                        payload: _,
                    }) => {
                        is_published = true;
                    }
                    rumqttc::Packet::PubAck(PubAck { pkid: _ }) if is_published => {
                        break;
                    }
                    _ => {}
                },
                _ => continue,
            }
        }
    }

    fn send_msg_mock(msg: &str) {
        println!("MQTT Mock message : {}", msg);
    }

    pub async fn handle_day(&self, tomorrow: &TempoDay) {
        self.send_msg(tomorrow.to_mqtt_string().as_str()).await;
        // Self::send_msg_mock(tomorrow.to_mqtt_string().as_str());
    }
}
