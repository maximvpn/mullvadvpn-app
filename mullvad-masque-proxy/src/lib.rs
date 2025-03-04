use bytes::{Buf, BufMut, Bytes, BytesMut};
use h3::{proto::varint::VarInt, quic::StreamId};
use h3_datagram::datagram_traits::HandleDatagramsExt;

pub mod client;
mod fragment;
pub mod server;

const PACKET_BUFFER_SIZE: usize = 1700;
pub const HTTP_MASQUE_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(0);
pub const HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID: VarInt = VarInt::from_u32(1);

// When a packet is larger than u16::MAX, it can't be fragmented.
#[derive(Debug)]
struct PacketTooLarge(usize);

async fn fragment_outgoing_packet(
    connection: &mut h3::client::Connection<h3_quinn::Connection, Bytes>,
    maximum_packet_size: u16,
    mut payload: Bytes,
    packet_id: u16,
    stream_id: StreamId,
) -> Result<(), PacketTooLarge> {
    payload.advance(1);
    let num_fragments: usize = payload.len() / usize::from(maximum_packet_size);
    let Ok(total_fragments): std::result::Result<u8, _> = num_fragments.try_into() else {
        return Err(PacketTooLarge(payload.len()));
    };

    for (fragment_index, fragment_payload) in payload.chunks(maximum_packet_size.into()).enumerate()
    {
        let mut fragment = BytesMut::with_capacity((maximum_packet_size + 1).into());
        crate::HTTP_MASQUE_FRAGMENTED_DATAGRAM_CONTEXT_ID.encode(&mut fragment);
        fragment.put_u16(packet_id);
        fragment.put_u8(u8::try_from(fragment_index).map_err(|_| PacketTooLarge(payload.len()))?);
        fragment.put_u8(total_fragments);
        fragment.extend_from_slice(fragment_payload);
        connection.send_datagram(stream_id, fragment.into());
    }

    Ok(())
}
