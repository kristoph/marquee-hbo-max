use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

const LOOPBACK_ANY_PORT: &str = "127.0.0.1:0";
const PLAYLIST_TYPE: &str = "application/vnd.apple.mpegurl";

/// The system's player only takes playlists from an address, so the ones written here are
/// served to it from this computer alone, for as long as the title is open.
pub struct PlaylistServer {
    address: SocketAddr,
    stopping: Arc<AtomicBool>,
}

impl PlaylistServer {
    pub fn serve(playlists: HashMap<String, String>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(LOOPBACK_ANY_PORT)?;
        let address = listener.local_addr()?;
        let stopping = Arc::new(AtomicBool::new(false));
        let stop = stopping.clone();
        thread::spawn(move || {
            for connection in listener.incoming().flatten() {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                let _ = answer(connection, &playlists);
            }
        });
        Ok(Self { address, stopping })
    }

    pub fn url(&self, playlist: &str) -> String {
        format!("http://{}/{playlist}", self.address)
    }
}

impl Drop for PlaylistServer {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Relaxed);
        // The server thread is waiting for a connection; this one lets it notice it should stop.
        let _ = TcpStream::connect(self.address);
    }
}

fn answer(mut connection: TcpStream, playlists: &HashMap<String, String>) -> std::io::Result<()> {
    let mut request_line = String::new();
    BufReader::new(&connection).read_line(&mut request_line)?;
    let name = requested_name(&request_line).unwrap_or_default();
    match playlists.get(name) {
        Some(playlist) => write!(
            connection,
            "HTTP/1.1 200 OK\r\ncontent-type: {PLAYLIST_TYPE}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{playlist}",
            playlist.len()
        ),
        None => write!(connection, "HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"),
    }
}

fn requested_name(request_line: &str) -> Option<&str> {
    let path = request_line.strip_prefix("GET /")?.split_whitespace().next()?;
    path.split('?').next()
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn reads_the_playlist_name_from_a_request() {
        assert_eq!(requested_name("GET /master.m3u8 HTTP/1.1\r\n"), Some("master.m3u8"));
        assert_eq!(requested_name("GET /video-0.m3u8?x=1 HTTP/1.1\r\n"), Some("video-0.m3u8"));
        assert_eq!(requested_name("POST /master.m3u8 HTTP/1.1\r\n"), None);
    }

    #[test]
    fn serves_its_playlists_and_nothing_else() {
        let server = PlaylistServer::serve(HashMap::from([("master.m3u8".to_string(), "#EXTM3U\n".to_string())])).unwrap();
        let fetch = |name: &str| {
            let mut connection = TcpStream::connect(server.address).unwrap();
            write!(connection, "GET /{name} HTTP/1.1\r\nhost: x\r\n\r\n").unwrap();
            let mut answer = String::new();
            connection.read_to_string(&mut answer).unwrap();
            answer
        };
        assert!(fetch("master.m3u8").ends_with("\r\n\r\n#EXTM3U\n"));
        assert!(fetch("other.m3u8").starts_with("HTTP/1.1 404"));
    }
}
