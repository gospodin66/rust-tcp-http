use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;
use std::io::Write;
use chrono::Local;
//use url::{Url, ParseError};

use crate::server::cstmfiles;
use crate::server::cstmconfig::{AssetsConfig, ServerConfig, BaseConfig};
use crate::server::headers;
use crate::server::database;

fn validate_request_method(meth: &str) -> Result<(), String> {
    let server_config: ServerConfig = ServerConfig::new_cfg();
    for method in server_config.request_methods {
        if meth == method {
            return Ok(());
        }
    }
    Err(String::from("http-response: Invalid request method."))
}
fn validate_route<'a>(route: &'a str, routes: &Vec<&str>) -> Result<(), String> {
    for (_, r) in routes.iter().enumerate() {
        if <String as From<&str>>::from(r) == String::from(route) {
            return Ok(());
        }
    }
    Err(String::from("http-response: Invalid route path."))
}

fn fetch_get_routes() -> Vec<&'static str> {
    vec![
        "/",
        "/users",
        "/tokens",
        "/tables"
    ]
}
fn fetch_post_routes() -> Vec<&'static str> {
    vec![
        "/",
        "/users",
        "/tokens",
        "/tables"
    ]
}
fn fetch_empty_routes() -> Vec<&'static str> {
    return vec![]
}


fn build_http_response(buffer: &str) -> Result<(&str,&str,Vec<&str>,String,String), String> {
    let get_routes = |req_method: &str| -> Vec<&str> {
        match validate_request_method(&req_method) {
            Ok(()) => {
                if req_method == "GET" {
                    fetch_get_routes()
                } else if req_method == "POST" {
                    fetch_post_routes()
                } else {
                    fetch_empty_routes()
                }
            },
            Err(_) => {
                fetch_empty_routes()
            }
        }
    };
    let res_ok : String = format!("{} 200 OK", BaseConfig::new_cfg().http_protocol);
    let assets_cfg: AssetsConfig = AssetsConfig::new_cfg();
    let http_req : Vec<&str>;
    /*
     * use 1st tuple val of buffer, drop the rest as
     * req_method|route|http_proto are always first in HTTP request
     */
    match validate_http_request(&buffer) {
        Ok(http_request_parsed) => {
            http_req = http_request_parsed;
        },
        Err(e) => {
            return Err(format!("http-response: Error validating HTTP request: {}", e));
        }
    }

    let req_method: &str = http_req[0];
    let req_route: &str = http_req[1];
    let routes: Vec<&str> = get_routes(req_method);

    match validate_route(&req_route, &routes) {
        Ok(()) => {},
        Err(e) => {
            return Err(format!("http-response: Error validating HTTP route: {}", e));
        }
    }

    let (status_line, view_file) = if req_route == routes[0] {
        (res_ok, format!("{}page.html", assets_cfg.html_base_path))
    } else if req_route == routes[1] {
        (res_ok, format!("{}users.html", assets_cfg.html_base_path))
    } else if req_route == routes[2] {
        (res_ok, format!("{}tokens.html", assets_cfg.html_base_path))
    } else {
        (res_ok, format!("{}notfound.html", assets_cfg.html_base_path))
    };

    Ok((req_method, req_route, routes, status_line, view_file))
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

pub fn write_http_response(mut stream: &TcpStream, buffer: &str) -> Result<(), String> {
    let mut response_data: String = String::new();
    let mut response: String = String::new();
    let mut contents_all: String = String::new();
    let (req_method, route, routes, mut status_line, view_file) : (&str,&str,Vec<&str>,String,String);
    let assets_cfg: AssetsConfig = AssetsConfig::new_cfg();
    let fpath: String = assets_cfg.log_dir+"/"+&assets_cfg.log_path;

    match build_http_response(&buffer) {
        Ok((rm, rt, rts, sl, vf)) => {
            req_method = rm;
            route = rt;
            routes = rts;
            status_line = sl;
            view_file = vf;
        },
        Err(e) => {
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            return Err(format!("http-response: Error building response: {}", e));
        }
    }

    match process_request(&req_method, &route, &routes, &buffer) {
        Ok(res_data) => {
            response_data = res_data;
        },
        Err(e) => {
            //stream.shutdown(std::net::Shutdown::Write).unwrap();
            println!("http-response: Error processing request: {}", e);
            status_line = format!("{} 500 Internal Server Error", BaseConfig::new_cfg().http_protocol);
            contents_all = String::from("500 Custom Server Error");
        }
    }

    // if error is reached before => don't read html page
    if contents_all.is_empty() {
        match fs::read_to_string(view_file) {
            Ok(contents) => {
                contents_all = format!("{}{}", contents, response_data);
            },
            Err(e) => {
                // file read error
                println!("http-response: Error opening content file: {}", e);
                contents_all = String::from("500 Custom Server Error");
                status_line = format!("{} 500 Internal Server Error", BaseConfig::new_cfg().http_protocol);
            }
        }
    }

    let headers: [String; 10] = headers::fetch_headers(contents_all.len());
    /*
     * HTTP text-based protocol basic response format:
     * {HTTP/1.1 200 OK}\r\n
     * {HEADERS}\r\n
     * {CONTENT}
     */
    response.push_str(format!(
        "{}\r\n{}\r\n\r\n{}",
        status_line,
        headers.iter().map(|val| val.to_string() + "\r\n").collect::<String>(),
        contents_all,
    ).as_str());

    match stream.write(response.as_bytes()) {
        Ok(bytes) => {
            let msg: String = format!(
                "[{}]: http-response: Successfuly written {} bytes to stream.",
                Local::now().to_rfc3339(),
                bytes
            );
            println!("{}", &msg);
            match cstmfiles::f_write(&fpath, msg) {
                Ok(()) => {}
                Err(e) => { println!("Oops! Error writing to log! {:?}", e); }
            }
        }, 
        Err(e) => {
            println!("http-response: Error writing to stream: {}", e);
        }
    }
    // flush() ensures all data is written on the stream
    match stream.flush() {
        Ok(()) => {}, 
        Err(e) => {
            println!("http-response: Error flushing stream: {}", e);
        }
    }
    Ok(())
}



pub fn parse_request_parameters(buffer: &str) -> HashMap<&str, &str> {

    let lines = buffer.split("\r\n").collect::<Vec<&str>>();

    match lines.last() {
        Some(element) => {
            let req_params = element.split('&').collect::<Vec<&str>>();
            let mut params = std::collections::HashMap::new();
            for param in req_params.iter() {
                let k_v = param.split_once('=').unwrap();
                params.insert(k_v.0, k_v.1);
            }
            params
        },        
        None => std::collections::HashMap::new()
    }
}




pub fn process_request(request_method: &str, route: &str, routes: &Vec<&str>, buffer: &str) -> Result<String, String>{
    let mut response_data : String = String::new();
    // SELECT on GET | INSERT on POST
    if request_method == "POST" {


        let params = parse_request_parameters(&buffer);
        println!("\n\nDEBUG: {:?}\n\n", params);

        
        if route == routes[1] {
            match database::User::create_users(params) {
                Ok(()) => {},
                Err(e) => {
                    let errmsg: String = format!("request: Error inserting users: {}", e);
                    println!("{}", &errmsg);
                    return Err(errmsg);
                }
            }
        }
        else if route == routes[2] {
            match database::Token::create_tokens(params) {
                Ok(()) => {},
                Err(e) => {
                    let errmsg: String = format!("request: Error inserting tokens: {}", e);
                    println!("{}", &errmsg);
                    return Err(errmsg);
                }
            }
        }
        else if route == routes[0] { /*** (default route '/') */
            response_data = String::from("Default route - default response :3")
        }
        else if route == routes[3] {
            match database::create_tables() {
                Ok(()) => {
                    response_data.push_str("Tables created successfuly!");
                },
                Err(e) => {
                    println!("SQL Error creating table: {}", e)
                }
            }
        }
        else {
            return Err(String::from(format!("request: Invalid route {}", route)))
        }
    }
    else if request_method == "GET" {
        if route == routes[1] {
            match database::User::select_all() {
                Ok(u) => {
                    let mut usersstr: String = String::new();
                    for user in u.iter() {
                        usersstr.push_str(database::User::user_to_string(&user).as_str());
                    }
                    response_data.push_str(usersstr.as_str());
                },
                Err(e) => {
                    println!("request: Error selecting users: {:?}", e);
                }
            }
        }
        else if route == routes[2] {
            match database::Token::select_all() {
                Ok(t) => {
                    let mut tokensstr = String::new();
                    for token in t.iter() {
                        tokensstr.push_str(database::Token::token_to_string(&token).as_str());
                    }
                    response_data.push_str(tokensstr.as_str());
                },
                Err(e) => {
                    println!("request: Error selecting tokens: {:?}", e);
                }
            }
        }
        else if route == routes[0] { /*** (default route '/') */
            response_data = String::from("Default route - default response :3")
        }
        else {
            return Err(String::from(format!("request: Invalid route {}", route)))
        }
    }
    else {
        return Err(String::from(format!("request: Invalid request method [{}]: supporting only GET|POST", request_method)))
    }

    Ok(response_data)
}
