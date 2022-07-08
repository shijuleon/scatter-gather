use configparser::ini::{Ini, IniDefault};
use ssh2::Session;
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;

struct Instance {
  connections: HashMap<String, Connection>,
  sessions: HashMap<String, Session>,
}

enum ConnectionState {
  Open,
  Closed,
}

struct Connection {
  state: ConnectionState,
}

impl Instance {
  fn new() -> Instance {
    Instance {
      connections: HashMap::new(),
      sessions: HashMap::new(),
    }
  }

  fn create_connection(mut self, addr: String) -> Self {
    self.connections.insert(
      addr.to_string(),
      Connection {
        state: ConnectionState::Open,
      },
    );
    self
  }
  fn create_session(&mut self, addr: String) {
    let mut sess = Session::new().unwrap();
    let tcp = TcpStream::connect(&addr).unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "test").unwrap();
    self.sessions.insert(addr, sess);
  }
}

fn exec(session: &Session) -> String {
  let mut channel = session.channel_session().unwrap();
  channel
    .exec("grep blog /root/access.log | awk '{print $4\" \"$5}'")
    .unwrap();
  let mut s = String::new();
  channel.read_to_string(&mut s).unwrap();
  channel.wait_close();
  s
}

fn main() {
  let mut config = Ini::new();
  let mut default = IniDefault::default();
  default.delimiters = vec!['='];
  config.load_defaults(default.clone());
  let hosts = config.load("tests/hosts").unwrap();

  let mut n = Instance::new();
  let mut screen: String = "".to_owned();
  for (host, _) in &hosts["hosts"] {
    // TODO: schedule to an event loop
    n = n.create_connection(host.to_string());
    n.create_session(host.to_string());
    let session = n.sessions.get(&host.to_string()).unwrap();
    screen.push_str(&exec(session));
  }

  println!("{}", screen);
}
