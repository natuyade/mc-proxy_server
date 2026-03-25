use crate::{parse_status_ping_payload, ConnectionContext};
use std::io::{Error, ErrorKind};

pub fn handle_status_packet(packet_id: i32, payload_slice: &[u8], _ctx: &ConnectionContext) -> std::io::Result<()> {
    
    match packet_id {
        // サーバーの稼働状況を返す
        0x00 => {

            // 0x00はサーバー一覧での表示用の情報だけがrequestされるのでpayloadがないのが正常
            if payload_slice.is_empty() {
            } else {
                return Err(Error::new(ErrorKind::InvalidData, format!("invalid Status Request payload length: expected 0, got {}", payload_slice.len())))
            }
            Ok(())
        }
        // サーバーの遅延状況(ping)を返す
        0x01 => {

            // 0x01はclient側からlatancy測定のために送信するpacket(payload: 8bytesLong)
            if payload_slice.len() == 8 {
            } else {
                return Err(Error::new(ErrorKind::InvalidData, format!("invalid Status Ping payload length: expected 8, got {}", payload_slice.len())))
            }

            let _payload = parse_status_ping_payload(payload_slice)?;
            Ok(())
        }
        _ => {
            Err(Error::new(ErrorKind::InvalidData, format!("unexpected packet in Status state: id = 0x{:02X}", packet_id)))
        }
    }
}