use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use log::{debug, error, info, trace};
use url::{Host, Url};

use crate::error::{Error, Result};

pub trait Input {
    fn open(&mut self) -> Result<()>;
    fn read(&self) -> Result<()>;
    fn close(&self) -> Result<()>;
}

type UDPFifo = Arc<(Mutex<VecDeque<[u8; ts::Packet::SZ]>>, Condvar)>;

pub struct InputUdp {
    url: Url,

    /// a.k.a. circular buffer size
    fifo_sz: usize,

    /// circullar-buffer / fifo
    /// use two threads and buffer to read from udp
    fifo: Option<UDPFifo>,
}

impl InputUdp {
    pub fn new(url: Url) -> InputUdp {
        InputUdp {
            url,
            fifo_sz: 1000,
            fifo: None,
        }
    }

    pub fn fifo_sz(&mut self, fifo_sz: usize) -> &InputUdp {
        self.fifo_sz = fifo_sz;
        self
    }
}

impl Input for InputUdp {
    fn open(&mut self) -> Result<()> {
        let fifo = Arc::new((
            Mutex::new(VecDeque::with_capacity(self.fifo_sz)),
            Condvar::new(),
        ));
        self.fifo = Some(fifo.clone());

        let host = self.url.host().ok_or_else(Error::udp_url_missing_host)?;
        let host_str = host.to_owned().to_string();

        let port = self.url.port().unwrap_or(5500);

        let socket = UdpSocket::bind((&*host_str, port))
            .map_err(|err| Error::udp_socket_bind(err, &host_str, port))?;

        debug!("({}) [+] OK bind udp socket", self.url);

        {
            match host {
                Host::Ipv4(v4) => {
                    let iface = Ipv4Addr::new(0, 0, 0, 0);
                    socket.join_multicast_v4(&v4, &iface).map_err(|err| {
                        Error::udp_join_multicast_v4(err, host_str, port, iface.to_string())
                    })?;

                    debug!("({}) [+] OK join multicast v4", self.url);
                    debug!("({}) [+] OK ({}:{}@{})", self.url, v4, port, iface);
                }
                Host::Ipv6(v6) => {
                    // 0 to indicate any interface
                    let iface = 0;
                    socket
                        .join_multicast_v6(&v6, iface)
                        .map_err(|err| Error::udp_join_multicast_v6(err, host_str, port, iface))?;

                    debug!("({}) [+] OK join multicast v6", self.url);
                }
                Host::Domain(domain) => {
                    let v4 = domain
                        .parse()
                        .map_err(|err| Error::udp_domain_to_ipv4(err, domain))?;

                    let iface = Ipv4Addr::new(0, 0, 0, 0);
                    socket.join_multicast_v4(&v4, &iface).map_err(|err| {
                        Error::udp_join_multicast_v4(err, host_str, port, iface.to_string())
                    })?;

                    debug!("({}) [+] OK join multicast v4/domain", self.url);
                    debug!("({}) [+] OK ({}:{}@{})", self.url, domain, port, iface);
                }
            }
        }

        let url = self.url.clone();
        thread::spawn(move || {
            let mut pkt_raw = [0; ts::Packet::SZ];

            // MTU (maximum transmission unit) == 1500 for Ethertnet
            // 7*ts::Packet::SZ = 7*188 = 1316 < 1500 => OK
            let mut buf7 = [0; 7 * ts::Packet::SZ];

            loop {
                let (_, _) = socket.recv_from(&mut buf7).unwrap();

                let &(ref lock, ref cvar) = &*fifo;
                let mut fifo = match lock.lock() {
                    Err(e) => {
                        error!("({}) lock and get buffer failed: {}", url, e);
                        // will retry after timeout;
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                    Ok(buf) => buf,
                };

                for i in 0..7 {
                    let f = i * ts::Packet::SZ;
                    let t = (i + 1) * ts::Packet::SZ;
                    let buf1 = &buf7[f..t];

                    pkt_raw.copy_from_slice(buf1);
                    fifo.push_back(pkt_raw);
                }

                cvar.notify_all();
            }
        });

        Ok(())
    }
    fn read(&self) -> Result<()> {
        let fifo = self
            .fifo
            .as_ref()
            .ok_or_else(Error::udp_fifo_not_initialized)?
            .clone();

        let &(ref lock, ref cvar) = &*fifo;
        let mut fifo = lock
            .lock()
            .map_err(|err| Error::udp_fifo_lock(err.to_string()))?;

        fifo = cvar
            .wait(fifo)
            .map_err(|err| Error::udp_fifo_cvar_wait(err.to_string()))?;

        while !fifo.is_empty() {
            let ts_pkt_raw = fifo.pop_front().ok_or_else(Error::udp_fifo_pop_empty)?;
            trace!("({}) [<] {}", self.url, ts_pkt_raw.len())
        }

        Ok(())
    }
    fn close(&self) -> Result<()> {
        Ok(())
    }
}

pub struct InputFile {
    url: Url,
}

impl InputFile {
    pub fn new(url: Url) -> InputFile {
        InputFile { url }
    }
}

impl Input for InputFile {
    fn open(&mut self) -> Result<()> {
        Ok(())
    }
    fn read(&self) -> Result<()> {
        info!("[<] {}", self.url);
        Ok(())
    }
    fn close(&self) -> Result<()> {
        Ok(())
    }
}
