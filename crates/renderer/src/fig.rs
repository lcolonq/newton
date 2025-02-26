use std::io::{BufRead, Write};

#[derive(Debug, Clone)]
pub struct Message {
    pub event: lexpr::Value,
    pub data: lexpr::Value,
}

pub struct Client {
    reader: std::io::BufReader<std::net::TcpStream>,
    buf: String,
}
impl Client {
    pub fn new(addr: &str, subs: &[lexpr::Value]) -> Self {
        let mut socket = std::net::TcpStream::connect(addr).expect("failed to connect to message bus");
        socket.set_nonblocking(true).expect("failed to set message bus socket nonblocking");
        for s in subs {
            write!(socket, "(sub {})\n", s).expect("failed to send subscribe message to bus");
        }
        let reader = std::io::BufReader::new(socket);
        Self { reader, buf: String::new(), }
    }
    pub fn pump(&mut self) -> Option<Message> {
        match self.reader.read_line(&mut self.buf) {
            Ok(l) => {
                // log::info!("read line: {}", self.buf);
                let mv = lexpr::from_str(&self.buf);
                self.buf.clear();
                match mv {
                    Ok(v) => {
                        match v.as_cons() {
                            Some(cs) => {
                                Some(Message { event: cs.car().clone(), data: cs.cdr().clone() })
                            },
                            _ => { log::error!("malformed message bus input s-expression: {}", v); None },
                        }
                    },
                    Err(e) => { log::error!("malformed message bus input line: {}", e); None },
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if self.buf.len() > 0 {
                    log::error!("error wouldblock: buf is {}", self.buf);
                }
                None
            },
            Err(e) => panic!("IO error on message bus: {}", e),
        }
    }
}
