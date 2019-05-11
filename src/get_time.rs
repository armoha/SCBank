use ntp;
use time;

pub fn get_utc_tm() -> String {
    let address = "0.pool.ntp.org:123";
    let response: ntp::packet::Packet = ntp::request(address).unwrap();
    let ntp_time = response.transmit_time;
    let t = time::at_utc(time::Timespec::from(ntp_time));
    t.asctime().to_string()
}
