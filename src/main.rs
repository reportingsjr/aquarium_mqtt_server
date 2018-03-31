extern crate mqttc;
extern crate mqtt3;
extern crate netopt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[allow(unused_imports)]
use std::sync::Arc;
use netopt::NetworkOptions;
#[allow(unused_imports)]
use mqttc::{PubSub, Client, ClientOptions, ReconnectMethod};
use mqtt3::{Message, TopicPath, QoS, PacketIdentifier};

struct Temperature {
	temperature: f64,
	timestamp: Duration,
}

fn main() {
    let address = "192.168.0.22:1883";
    // Connect to broker, send CONNECT then wait CONNACK
    let netopt = NetworkOptions::new();
    let mut opts = ClientOptions::new();
    opts.set_keep_alive(15);
    opts.set_reconnect(ReconnectMethod::ReconnectAfter(Duration::new(5,0)));
    let mut client = opts.connect(address, netopt).unwrap();

    client.subscribe("/aquarium/temperature").unwrap();
    client.await().unwrap();
    loop {
        match client.await() {
            Ok(result) => {
                match result {
                    Some(message) => {
                    	match parse_temperature(message) {
                    		Ok(temperature) => println!("Timestamp: {:?}, Temperature: {:?}", temperature.timestamp.as_secs(), temperature.temperature),
                    		Err(_) => continue
                    	}
                    },
                    None => println!("."),
                }
            }
            Err(_) => continue
        }
    }
}

fn parse_temperature(message: Box<Message>) -> Result<Temperature, &'static str> {
	let raw_message = String::from_utf8(Arc::try_unwrap(message.payload).unwrap()).expect("invalid utf-8");
	let temperature = raw_message.parse::<f64>().unwrap();

	let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
		Ok(current_time) => current_time,
		Err(_) => return Err("failed to get current time")
	};

	if temperature < 0.0  {
		return Err("Temperature was outside of known good range.");
	}

	Ok(Temperature { 
		temperature: temperature,
		timestamp: time,
	})
}

#[test]
fn temperature_parsing() {
	let message = Box::new(Message {
		topic: TopicPath::from_str("/aquarium/temperature").unwrap(),
		qos: QoS::AtLeastOnce,
		retain: false,
		pid: Some(PacketIdentifier(1)),
		payload: Arc::new("20.0".as_bytes().to_vec())
	});

	let parsed_temp = parse_temperature(message).unwrap();

	assert_eq!(parsed_temp.temperature, 20.0);
}

#[test]
#[should_panic]
fn temperature_parsing_out_of_bounds() {
	let message = Box::new(Message {
		topic: TopicPath::from_str("/aquarium/temperature").unwrap(),
		qos: QoS::AtLeastOnce,
		retain: false,
		pid: Some(PacketIdentifier(1)),
		payload: Arc::new("-127.0".as_bytes().to_vec())
	});

	parse_temperature(message).unwrap();
}