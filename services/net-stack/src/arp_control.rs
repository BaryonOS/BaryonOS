use crate::{AddressConfiguration, Error, Stack};
use smoltcp::wire::{
    ArpOperation, ArpPacket, ArpRepr, EthernetAddress, EthernetFrame, EthernetProtocol,
    HardwareAddress, Ipv4Address,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArpObservation {
    Unrelated,
    OwnFrame,
    ForeignClaim,
    ForeignProbe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressArp {
    Probe,
    Announcement,
}

impl Stack<'_> {
    pub fn hardware_address(&self) -> [u8; 6] {
        let HardwareAddress::Ethernet(mac) = self.interface.hardware_addr();
        mac.0
    }

    pub fn can_queue_control_frame(&self) -> bool {
        self.device.transmission().is_none()
    }

    /// RFC5227 control bytes, not a completion or authorization claim.
    pub fn queue_address_arp(
        &mut self,
        address: [u8; 4],
        kind: AddressArp,
        ticks: u64,
    ) -> Result<u64, Error> {
        AddressConfiguration {
            ipv4: address,
            prefix_len: 32,
            gateway: None,
        }
        .validate()?;
        let address = Ipv4Address::from(address);
        let mac = EthernetAddress(self.hardware_address());
        self.queue_arp_packet(
            EthernetAddress::BROADCAST,
            ArpRepr::EthernetIpv4 {
                operation: ArpOperation::Request,
                source_hardware_addr: mac,
                source_protocol_addr: match kind {
                    AddressArp::Probe => Ipv4Address::UNSPECIFIED,
                    AddressArp::Announcement => address,
                },
                target_hardware_addr: EthernetAddress([0; 6]),
                target_protocol_addr: address,
            },
            ticks,
        )
    }

    fn queue_arp_packet(
        &mut self,
        destination: EthernetAddress,
        representation: ArpRepr,
        ticks: u64,
    ) -> Result<u64, Error> {
        let mut bytes = [0; 60];
        let mut frame = EthernetFrame::new_unchecked(&mut bytes[..]);
        frame.set_src_addr(EthernetAddress(self.hardware_address()));
        frame.set_dst_addr(destination);
        frame.set_ethertype(EthernetProtocol::Arp);
        representation.emit(&mut ArpPacket::new_unchecked(frame.payload_mut()));
        self.queue_control_frame(&bytes, ticks)
    }

    /// Upstream's neighbor path rejects zero-source requests. RFC5227 section
    /// 2.5 still requires a reply, without caching zero as a neighbor address.
    pub fn queue_address_probe_reply(&mut self, bytes: &[u8], ticks: u64) -> Result<u64, Error> {
        let address = self.configuration.ok_or(Error::NotConfigured)?.ipv4;
        if self.observe_address_arp(bytes, address)? != ArpObservation::ForeignProbe {
            return Err(Error::InvalidFrame);
        }
        let frame = EthernetFrame::new_checked(bytes).map_err(|_| Error::InvalidFrame)?;
        let destination = frame.src_addr();
        self.queue_arp_packet(
            destination,
            ArpRepr::EthernetIpv4 {
                operation: ArpOperation::Reply,
                source_hardware_addr: EthernetAddress(self.hardware_address()),
                source_protocol_addr: Ipv4Address::from(address),
                target_hardware_addr: destination,
                target_protocol_addr: Ipv4Address::UNSPECIFIED,
            },
            ticks,
        )
    }

    /// Observe requests AND replies, including a probe response whose target
    /// IPv4 is zero. The initial neighbor codec intentionally rejects those.
    pub fn observe_address_arp(
        &self,
        bytes: &[u8],
        address: [u8; 4],
    ) -> Result<ArpObservation, Error> {
        if bytes.len() != 42 && bytes.len() != 60 {
            return Err(Error::InvalidFrame);
        }
        let frame = EthernetFrame::new_checked(bytes).map_err(|_| Error::InvalidFrame)?;
        if frame.ethertype() != EthernetProtocol::Arp {
            return Err(Error::InvalidFrame);
        }
        let packet = ArpPacket::new_checked(frame.payload()).map_err(|_| Error::InvalidFrame)?;
        let ArpRepr::EthernetIpv4 {
            operation,
            source_hardware_addr,
            source_protocol_addr,
            target_protocol_addr,
            ..
        } = ArpRepr::parse(&packet).map_err(|_| Error::InvalidFrame)?
        else {
            return Err(Error::InvalidFrame);
        };
        if !matches!(operation, ArpOperation::Request | ArpOperation::Reply)
            || !source_hardware_addr.is_unicast()
            || source_hardware_addr.0 == [0; 6]
            || frame.src_addr() != source_hardware_addr
        {
            return Err(Error::InvalidFrame);
        }
        if source_hardware_addr.0 == self.hardware_address() {
            return Ok(ArpObservation::OwnFrame);
        }
        if source_protocol_addr.octets() == address {
            return Ok(ArpObservation::ForeignClaim);
        }
        if operation == ArpOperation::Request
            && source_protocol_addr.is_unspecified()
            && target_protocol_addr.octets() == address
        {
            return Ok(ArpObservation::ForeignProbe);
        }
        Ok(ArpObservation::Unrelated)
    }
}
