use rspotify::{oauth2::SpotifyOAuth, util::request_token};
use std::{
  io::prelude::*,
  net::{Ipv4Addr, TcpListener, TcpStream},
};

pub fn redirect_uri_web_server(spotify_oauth: &mut SpotifyOAuth, port: u16) -> Result<String, ()> {
  match TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
    Ok(listener) => {
      request_token(spotify_oauth);

      for stream in listener.incoming() {
        match stream {
          Ok(stream) => {
            if let Some(url) = handle_connection(stream) {
              return Ok(url);
            }
          }
          Err(e) => {
            println!("Error: {}", e);
          }
        };
      }
    }
    Err(e) => {
      println!("Error: {}", e);
    }
  }

  Err(())
}

fn handle_connection(mut stream: TcpStream) -> Option<String> {
  // The request will be quite large (> 512) so just assign plenty just in case
  let mut buffer = [0; 1000];
  stream.read(&mut buffer).unwrap();

  // convert buffer into a str and 'parse' the 'URL' to get at the query paramaters
  match std::str::from_utf8(&buffer) {
    // index 0    1                 2            ....
    //       GET /redirect?xxxxxxxx HTTP/1.1\r\n ....
    Ok(request) => match request.split_whitespace().nth(1) {
      Some(url) => {
        respond_with_success(stream);
        return Some(url.to_string());
      }
      None => respond_with_error("Malformed request".to_string(), stream),
    },
    Err(e) => {
      respond_with_error(format!("Invalid UTF-8 sequence: {}", e), stream);
    }
  };

  None
}


fn respond_with_success(mut stream: TcpStream) {
  let contents = include_str!("redirect_uri.html");

  write!(stream, "HTTP/1.1 200 OK\r\n\r\n{}", contents).unwrap();
}

fn respond_with_error(error_message: String, mut stream: TcpStream) {
  println!("Error: {}", error_message);
  write!(
    stream,
    "HTTP/1.1 400 Bad Request\r\n\r\n400 - Bad Request - {}",
    error_message
  )
  .unwrap();
}
