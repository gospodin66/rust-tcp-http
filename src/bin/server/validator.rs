use crate::server::cstmconfig::ServerConfig;

pub fn validate_request_method(request_method: &str) -> Result<(), String> {
    let server_config: ServerConfig = ServerConfig::new_cfg();
    for method in server_config.request_methods {
        if request_method == method {
            return Ok(());
        }
    }
    Err(String::from("http-response: Invalid request method."))
}
pub fn validate_route<'a>(route: &'a str, routes: &Vec<&str>) -> Result<(), String> {
    for (_, r) in routes.iter().enumerate() {
        if <String as From<&str>>::from(r) == String::from(route) {
            return Ok(());
        }
    }
    Err(String::from("http-response: Invalid route path."))
}
pub fn validate_http_request(buffer: &str) -> Result<Vec<&str>, String> {
    match buffer.split_once("\r\n") {
        Some(httprequest) => {
            let http_req : Vec<&str> = httprequest.0.split(' ').collect();
            Ok(http_req)
        }, 
        None => {
            // ???????
            let errmsg: &str = "request: Input does not consist of any newlines - not a HTTP request - skipping..";
            return Err(String::from(errmsg))
        }
    }
}