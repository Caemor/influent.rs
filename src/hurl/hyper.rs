
extern crate reqwest;



use hurl::hyper::reqwest::header::Basic;
use hurl::hyper::reqwest::header::Authorization;




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

/*

        let request = Request {
            url: &*{host.to_string() + "/query"},
            method: Method::GET,
            auth: Some(Auth {
                username: self.credentials.username,
                password: self.credentials.password
            }),
            query: Some(query),
            body: None
        };

*/

impl Hurl for HyperHurl {
    fn request(&self, req: Request) -> HurlResult {      

        

        // map request method to the hyper's
        let method = match req.method {
            Method::POST => self::reqwest::Method::Post,
            Method::GET  => self::reqwest::Method::Get
        };

        

        let mut url = match self::reqwest::Url::parse(req.url) {
            Ok(u) => { u }
            Err(e) => {
                return Err(format!("could not parse url: {:?}", e));
            }
        };


        let mut headers = self::reqwest::header::Headers::new();
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

        let req_client = self::reqwest::Client::builder()
            .danger_disable_hostname_verification()
            .build()
            .expect("Client builder failed"); //TODO fix for valid certificates, but this is needed for self signed ones




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

        //let uri: Uri = url.parse().expect("parse url");

        /*let mut hyperrequest = HyperRequest::builder()
            //.method(method)
            //.uri(uri)
            //.header(headers);
            .body(req.body.expect("missing body"))
            .expect("Building Hyper Request failed");*/
            


        // create query
        //let mut query = client.request(method, url).headers(headers);
        let mut query = req_client.request(method, url);
        
        query.headers(headers);

        // if request has body
        match req.body {
            Some(ref body) => {
                query.body(body.clone().to_string());
            }
            None => { }
        }

        
        

        //let mut query = client.request(hyperrequest);

        //let mut query = 

        // go!
        match query.send() {
            Ok(ref mut resp) => {
                let mut body = String::new();
                resp.read_to_string(&mut body).unwrap();

                Ok(Response {
                    status: resp.status().as_u16(),
                    body: body
                })
            }
            Err(err) => {
                Err(format!("something went wrong: {:?}", err))
            }
        }
    }
}
