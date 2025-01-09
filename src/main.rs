pub mod ed_newtype;

#[macro_use] extern crate tracing;

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_elasticbeanstalk::config::http::HttpResponse;
use aws_sdk_elasticbeanstalk::operation::describe_events::{DescribeEventsError, DescribeEventsOutput};
use aws_sdk_elasticbeanstalk::types::{EventDescription, EventSeverity};
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use tracing_subscriber::EnvFilter;
use aws_smithy_types_convert::date_time::DateTimeExt;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::Duration;
use aws_sdk_elasticbeanstalk::primitives::DateTime as aws_datetime;
use tokio::time::sleep;
use crate::ed_newtype::MyEventDescription;
use std::env;
use dotenv::dotenv;
use std::collections::VecDeque;
use reqwest::{ClientBuilder, Error, Response, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use base64::prelude::*;
use std::str::FromStr;
use serde_json::Value;

pub const DEFAULT_REGION: &str = "us-west-2";



#[tokio::main]
async fn main() {

    let filter_layer =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info")).unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(filter_layer)
        .with_file(true)
        .with_line_number(true)
        .init();

    dotenv();

    // are we in a shell with aws access?
    let aws_key = env::var("AWS_ACCESS_KEY_ID").ok();
    if aws_key.is_none() {
        error!("You must pass in the AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY env vars to use this tool.");
        std::process::exit(1);
    };

    let logger_url = env::var("EB_EVENT_LOGGER_URL").ok();
    if logger_url.is_none() {
        error!("You must specify EB_EVENT_LOGGER_URL to send logs somewhere");
        std::process::exit(1);
    };

    let logger_user = env::var("EB_EVENT_LOGGER_USER").ok();
    let logger_password = env::var("EB_EVENT_LOGGER_PASSWORD").ok();
    let mut headers: HeaderMap = HeaderMap::new();
    if logger_user.is_some()  {
        if logger_password.is_none(){
            error!("You must specify EB_EVENT_LOGGER_PASSWORD if you specify EB_EVENT_LOGGER_USER");
            std::process::exit(1);
        }
        let raw_auth: String = format!("{}:{}",logger_user.unwrap(),logger_password.unwrap());
        let b64: String = BASE64_STANDARD.encode(raw_auth.as_str());
        let auth_str = format!("Basic {b64}");
        headers.insert(HeaderName::from_str("Authorization").unwrap(),HeaderValue::from_str(&auth_str).unwrap());
    }

    let logger_client = ClientBuilder::new().default_headers(headers).use_rustls_tls().build().expect("Could not build reqwest client");

    let region_provider = RegionProviderChain::default_provider().or_else(DEFAULT_REGION);
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let client = aws_sdk_elasticbeanstalk::Client::new(&aws_config);
    let mut last_msg: HashMap<String, aws_datetime> = HashMap::new();
    let mut msg_queue: VecDeque<String> = VecDeque::new();

    loop {
        let req = client.describe_environments().send().await;
        if let Ok(envs) = req {
            for env in envs.environments() {
                let env_name = env.environment_id().unwrap();
                if !last_msg.contains_key(env_name) {
                    last_msg.insert(env_name.parse().unwrap(), aws_datetime::from_chrono_utc(chrono::Utc::now()));
                }
            }
        }
        for (env_name,start_time) in last_msg.clone().iter() {
            info!("Querying for latest events in {} since {}", &env_name, start_time);
            let mut stream = client.describe_events().start_time((*start_time).into()).environment_id(env_name).into_paginator().send();
            match stream.try_next().await {
                Ok(maybe_event) => {
                    if let Some(output) = maybe_event {
                        let m = output.events.unwrap();
                        for event in m {
                            let mut timestamp: aws_datetime = event.event_date.clone().unwrap();
                            let med: MyEventDescription = MyEventDescription(event);
                            if let Ok(mut val) = serde_json::to_value(&med) {
                                let obj = val.as_object_mut().unwrap();
                                obj.insert("type".to_string(),
                                           Value::String("beanstalk-events".to_string())
                                );
                                let val_text = serde_json::to_string(&obj).unwrap();
                                msg_queue.push_front(val_text);
                            } else {
                                error!("Couldn't deserialize the event from AWS, tossing it out.");
                            }

                            if timestamp > *start_time {
                                let mut ts = timestamp.secs();
                                ts+=1;
                                timestamp.set_seconds(ts);
                                last_msg.insert(String::from(env_name.clone()), timestamp);
                            }

                        }
                    }
                }
                Err(e) => {
                    error!("{e:#?}");
                }
            }
        }
        let mut q = msg_queue.clone();
        for (idx, line) in msg_queue.iter().enumerate() {
            info!("processing queue");
            match logger_client.post(logger_url.clone().unwrap()).body(line.clone()).send().await {
                Ok(rs) => {
                    if rs.status() == StatusCode::OK {
                        let _ = q.remove(idx);
                        info!("Event emitted successfully");

                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            }
        }
        msg_queue = q;


        sleep(Duration::from_secs(30)).await;
    }
}
