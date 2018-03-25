extern crate mqttc;
extern crate netopt;
use std::process::exit;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use netopt::NetworkOptions;
use mqttc::{PubSub, Client, ClientOptions, ReconnectMethod, PubOpt};

fn main() {
    let address = "192.168.0.22:883";
    // Connect to broker, send CONNECT then wait CONNACK
    let netopt = NetworkOptions::new();
    let mut opts = ClientOptions::new();
    opts.set_keep_alive(15);
    opts.set_reconnect(ReconnectMethod::ReconnectAfter(Duration::new(5,0)));
    let mut client = opts.connect(address.as_str(), netopt).unwrap();

    client.subscribe("/aquarium/temperature").unwrap();
    client.await().unwrap();
    //client.subscribe("a/b/c").unwrap();
    //client.publish(topic.as_str(), "Hello", PubOpt::at_least_once());

    loop {
        match client.await() {
            Ok(result) => {
                match result {
                    Some(message) => {
                    	let message = String::from_utf8(Arc::try_unwrap(message.payload).unwrap()).expect("invalid utf-8");
                    	let temperature = message.parse::<f64>().unwrap();
                    	if(temperature < 0.0) {
                    		println!("Bad temperature reading: {:?}", temperature);
                    	} else {
                    		println!("Good temperature reading:{:?} {:?}", SystemTime::now().duration_since(UNIX_EPOCH).expect("failed to get current time").as_secs(), temperature);
                    	}
                    },
                    None => println!("."),
                }
            }
            Err(_) => continue
        }
    }
}