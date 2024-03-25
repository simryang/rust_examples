use std::net::{TcpStream, SocketAddr};
use std::io::{self, Read, Write};
use std::time::{SystemTime, Duration};

const NTP_SERVER: &str = "pool.ntp.org";
const NTP_PORT: u16 = 123;
const NTP_PACKET_SIZE: usize = 48;

// SNTP packet structure
#[repr(C)]
struct NtpPacket {
    flags: u8,
    stratum: u8,
    poll: u8,
    precision: u8,
    root_delay: u32,
    root_dispersion: u32,
    ref_id: u32,
    ref_timestamp: u64,
    orig_timestamp: u64,
    recv_timestamp: u64,
    trans_timestamp: u64,
}

impl NtpPacket {
    fn new() -> NtpPacket {
        NtpPacket {
            flags: 0b00100011,
            stratum: 0,
            poll: 4,
            precision: 0,
            root_delay: 0,
            root_dispersion: 0,
            ref_id: 0,
            ref_timestamp: 0,
            orig_timestamp: 0,
            recv_timestamp: 0,
            trans_timestamp: 0,
        }
    }
}

fn main() -> io::Result<()> {
    // Connect to NTP server
    let ntp_addr = format!("{}:{}", NTP_SERVER, NTP_PORT);
    let addr: SocketAddr = ntp_addr.parse()?;
    let mut stream = TcpStream::connect(addr)?;

    // Create an SNTP request packet
    let mut request_packet = NtpPacket::new();
    let transmit_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    request_packet.trans_timestamp = transmit_time.as_secs() as u64;

    // Send request packet
    let request_bytes = unsafe {
        std::slice::from_raw_parts(
            &request_packet as *const _ as *const u8,
            std::mem::size_of::<NtpPacket>(),
        )
    };
    stream.write_all(request_bytes)?;

    // Receive response packet
    let mut response_bytes = [0u8; NTP_PACKET_SIZE];
    stream.read_exact(&mut response_bytes)?;

    // Parse response packet
    let response_packet: NtpPacket = unsafe {
        std::ptr::read(response_bytes.as_ptr() as *const NtpPacket)
    };

    // Calculate time offset
    let orig_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let recv_time = response_packet.recv_timestamp;
    let trans_time = response_packet.trans_timestamp;
    let offset = ((recv_time - orig_time) + (trans_time - orig_time)) / 2;

    // Adjust system time
    let adjusted_time = SystemTime::now() + Duration::from_secs(offset);
    println!("Adjusted Time: {:?}", adjusted_time);

    Ok(())
}
