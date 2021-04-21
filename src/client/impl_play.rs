use crate::client::{ChoadraClient, Play};
use crate::error::ChoadraResult;
use crate::protocol::play::s2c::S2CPlayPacket;
use crate::protocol::play::c2s::C2SPlayPacket;

impl ChoadraClient<Play> {
    pub fn read_play_packet(&mut self) -> ChoadraResult<S2CPlayPacket> {
        if let Some(packet) = self.state.packet_queue.pop_front() {
            // Return out of the queue first
            return Ok(packet);
        }
        let packet = self.read_s2c_packet()?;
        self.state.really_playing = true;
        Ok(packet)
    }

    pub fn send_play_packet(&mut self, packet: impl C2SPlayPacket) -> ChoadraResult<()> {
        // we need to wait for the first play packet before sending
        // we'll keep it for `read_play_packet`
        if !self.state.really_playing {
            let packet = self.read_s2c_packet()?;
            self.state.really_playing = true;
            self.state.packet_queue.push_back(packet);
        }
        self.write_c2s_packet(packet)?;

        Ok(())
    }
}
