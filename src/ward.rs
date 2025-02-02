use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::env;
use crate::{WebFingerPrint, WebFingerPrintLib};
use url::Url;


#[derive(Debug)]
pub struct RawData {
    pub url: Url,
    pub path: String,
    pub headers: reqwest::header::HeaderMap,
    pub status_code: reqwest::StatusCode,
    pub text: String,
}

pub async fn check(raw_data: &Arc<RawData>, fingerprint_lib: &WebFingerPrintLib, is_special: bool) -> HashMap<String, u32> {
    let mut futures = vec![];
    let mut web_name_set: HashMap<String, u32> = HashMap::new();
    if is_special {
        for fingerprint in fingerprint_lib.special.iter() {
            futures.push(what_web(raw_data.clone(), fingerprint));
        }
    } else {
        for fingerprint in fingerprint_lib.index.iter() {
            futures.push(what_web(raw_data.clone(), fingerprint));
        }
    }
    let results = join_all(futures).await;
    for res in results {
        let (is_match, match_web_fingerprint) = res;
        if is_match {
            web_name_set.insert(match_web_fingerprint.name.clone(), match_web_fingerprint.priority.clone());
        }
    }
    return web_name_set;
}

pub async fn what_web(raw_data: Arc<RawData>, fingerprint: &WebFingerPrint) -> (bool, &WebFingerPrint) {
    let mut default_result = (false, fingerprint);
    if fingerprint.status_code != 0 && raw_data.status_code.as_u16() != fingerprint.status_code {
        return default_result;
    }
    for (k, v) in &fingerprint.headers {
        if raw_data.headers.contains_key(k) {
            let is_match = format!("{:?}", raw_data.headers).to_lowercase().find(&v.to_lowercase());
            if is_match == None && v != "*" {
                return default_result;
            }
        } else {
            return default_result;
        }
    }
    for keyword in &fingerprint.keyword {
        if raw_data.text.find(&keyword.to_lowercase()) == None {
            return default_result;
        }
    }
    default_result.0 = true;
    let is_output = env::var("OUTPUT_MATCH").is_ok();
    if is_output {
        println!("Matching fingerprint{:?}", fingerprint);
    }
    return default_result;
}