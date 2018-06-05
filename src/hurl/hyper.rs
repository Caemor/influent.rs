
extern crate hyper;
extern crate hyper_native_tls;

use self::hyper::Client as HyperClient;
use self::hyper::method::Method as HyperMethod;
use self::hyper::Url;
use self::hyper::header::{Headers, Authorization, Basic};

use self::hyper::net::HttpsConnector;
use self::hyper_native_tls::NativeTlsClient;


use super::{Request, Response, Method, HurlResult};
use std::io::Read;

use super::Hurl;

#[derive(Default)]
pub struct HyperHurl;

impl HyperHurl {
    pub fn new() -> HyperHurl {
        HyperHurl::default()
    }
}

impl Hurl for HyperHurl {
    fn request(&self, req: Request) -> HurlResult {
        let mut core = tokio_core::reactor::Core::new().unwrap();
        let handle = core.handle();
        let https = hyper_tls::HttpsConnector::new(4, &handle).expect("https");

        let client = HyperClient::configure()
            .connector(https).build(&handle);

        // map request method to the hyper's
        let method = match req.method {
            Method::POST => HyperMethod::Post,
            Method::GET  => HyperMethod::Get
        };

        let mut headers = Headers::new();

        let mut url = match Url::parse(req.url) {
            Ok(u) => { u }
            Err(e) => {
                return Err(format!("could not parse url: {:?}", e));
            }
        };

        // if request need to be authorized
        if let Some(auth) = req.auth {
            headers.set(
               Authorization(
                   Basic {
                       username: auth.username.to_string(),
                       password: Some(auth.password.to_string())
                   }
               )
            );
        }

        // if request has query
        if let Some(ref query) = req.query {
            // if any existing pairs
            let existing: Vec<(String, String)> = url.query_pairs().map(|(a,b)| (a.to_string(), b.to_string())).collect();

            // final pairs
            let mut pairs: Vec<(&str, &str)> = Vec::new();

            // add first existing
            for pair in &existing {
                pairs.push((&pair.0, &pair.1));
            }

            // add given query to the pairs
            for (key, val) in query.iter() {
                pairs.push((key, val));
            }

            // set new pairs
            url.query_pairs_mut().clear().extend_pairs(
                pairs.iter().map(|&(k, v)| { (&k[..], &v[..]) })
            );
        }

        // create query
        let mut query = client.request(method, url).headers(headers);

        // if request has body
        query = match req.body {
            Some(ref body) => {
                query.body(body)
            }
            None => { query }
        };

        // go!
        match query.send() {
            Ok(ref mut resp) => {
                let mut body = String::new();
                resp.read_to_string(&mut body).unwrap();

                Ok(Response {
                    status: resp.status.to_u16(),
                    body: body
                })
            }
            Err(err) => {
                Err(format!("something went wrong: {:?}", err))
            }
        }
    }
}
