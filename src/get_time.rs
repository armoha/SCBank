use ntp;
use time;

pub fn get_utc_tm() -> String {
    let address = "0.pool.ntp.org:123";
    let response: ntp::packet::Packet = ntp::request(address).unwrap();
    let mut ntp_time = response.transmit_time;
    ntp_time.sec += 32400;
    let t = time::at_utc(time::Timespec::from(ntp_time));
    t.asctime().to_string()
}
