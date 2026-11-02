#![no_std]
#![forbid(unsafe_code)]
//! NET1: bounded IPC bytes, never authority or a network authentication claim.
pub const TAG: u64 = u64::from_le_bytes(*b"NETRPC01");
pub const VERSION: u16 = 1;
pub const SIZE: usize = 256;
pub const HEADER: usize = 48;
pub const DATA: usize = SIZE - HEADER;
const REPLY: u16 = 0x8000;
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation { OpenTcp=1, SendTcp=2, ReceiveTcp=3, CloseTcp=4, State=5, SendUdp=6, ReceiveUdp=7, Retire=8 }
impl Operation {
    fn decode(value:u16)->Result<Self,Error>{Ok(match value {1=>Self::OpenTcp,2=>Self::SendTcp,3=>Self::ReceiveTcp,4=>Self::CloseTcp,5=>Self::State,6=>Self::SendUdp,7=>Self::ReceiveUdp,8=>Self::Retire,_=>return Err(Error::Operation)})}
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status { Ok=0, Again=1, Malformed=2, Access=3, Stale=4, Replay=5, Busy=6, Closed=7, Timeout=8, Uncertain=9, Exhausted=10 }
impl Status {
    fn decode(value:u32)->Result<Self,Error>{Ok(match value {0=>Self::Ok,1=>Self::Again,2=>Self::Malformed,3=>Self::Access,4=>Self::Stale,5=>Self::Replay,6=>Self::Busy,7=>Self::Closed,8=>Self::Timeout,9=>Self::Uncertain,10=>Self::Exhausted,_=>return Err(Error::Status)})}
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionState { Closed=0, Connecting=1, Established=2, PeerClosed=3, Closing=4, TimedOut=5 }
impl ConnectionState {
    pub fn decode(value:u32)->Result<Self,Error>{Ok(match value {0=>Self::Closed,1=>Self::Connecting,2=>Self::Established,3=>Self::PeerClosed,4=>Self::Closing,5=>Self::TimedOut,_=>return Err(Error::Argument)})}
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error { Length, Magic, Version, Operation, Status, Argument, Reserved, Identity, Binding }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeaderFields { pub operation:Operation,pub request:u64,pub epoch:u64,pub connection:u64,pub argument:u32,pub address:[u8;4],pub port:u16 }
#[derive(Clone, Copy)]
pub struct Packet { bytes:[u8;SIZE] }
fn get16(bytes:&[u8],at:usize)->u16{u16::from_le_bytes(bytes[at..at+2].try_into().unwrap())}
fn get32(bytes:&[u8],at:usize)->u32{u32::from_le_bytes(bytes[at..at+4].try_into().unwrap())}
fn get64(bytes:&[u8],at:usize)->u64{u64::from_le_bytes(bytes[at..at+8].try_into().unwrap())}
impl HeaderFields {
    fn encode(self,status:Status,reply:bool)->[u8;SIZE]{let mut b=[0;SIZE];b[..4].copy_from_slice(b"NET1");b[4..6].copy_from_slice(&VERSION.to_le_bytes());b[6..8].copy_from_slice(&(self.operation as u16|if reply{REPLY}else{0}).to_le_bytes());b[8..16].copy_from_slice(&self.request.to_le_bytes());b[16..24].copy_from_slice(&self.epoch.to_le_bytes());b[24..32].copy_from_slice(&self.connection.to_le_bytes());b[32..36].copy_from_slice(&(status as u32).to_le_bytes());b[36..40].copy_from_slice(&self.argument.to_le_bytes());b[40..44].copy_from_slice(&self.address);b[44..46].copy_from_slice(&self.port.to_le_bytes());b}
}
impl Packet {
    pub fn request(header:HeaderFields,data:&[u8])->Result<Self,Error>{
        if header.request==0||header.epoch==0{return Err(Error::Identity)}
        let mut p=Self{bytes:header.encode(Status::Ok,false)};
        let length=match header.operation{Operation::SendTcp|Operation::SendUdp=>header.argument as usize,_=>0};
        if data.len()!=length||length>DATA{return Err(Error::Argument)}
        p.bytes[HEADER..HEADER+length].copy_from_slice(data);Self::decode_request(&p.bytes)
    }
    pub fn response(mut header:HeaderFields,status:Status,argument:u32,data:&[u8])->Self{
        header.argument=argument;let mut p=Self{bytes:header.encode(status,true)};
        if status==Status::Ok&&matches!(header.operation,Operation::ReceiveTcp|Operation::ReceiveUdp){assert_eq!(data.len(),argument as usize);assert!(data.len()<=DATA);p.bytes[HEADER..HEADER+data.len()].copy_from_slice(data)}else{assert!(data.is_empty())}p
    }
    fn decode(bytes:&[u8],reply:bool)->Result<Self,Error>{
        if bytes.len()!=SIZE{return Err(Error::Length)}if &bytes[..4]!=b"NET1"{return Err(Error::Magic)}if get16(bytes,4)!=VERSION{return Err(Error::Version)}
        let op=get16(bytes,6);if(op&REPLY!=0)!=reply{return Err(Error::Operation)}Operation::decode(op&!REPLY)?;Status::decode(get32(bytes,32))?;
        if bytes[46..48]!=[0,0]{return Err(Error::Reserved)}
        let mut out=[0;SIZE];out.copy_from_slice(bytes);Ok(Self{bytes:out})
    }
    pub fn decode_request(bytes:&[u8])->Result<Self,Error>{
        let p=Self::decode(bytes,false)?;let h=p.header();if h.request==0||h.epoch==0||p.status()!=Status::Ok{return Err(Error::Identity)}
        let data_length=match h.operation{Operation::SendTcp|Operation::SendUdp=>{if h.argument as usize>DATA{return Err(Error::Argument)}h.argument as usize},Operation::OpenTcp=>{if !(1..=6000).contains(&h.argument){return Err(Error::Argument)}0},_=>{if h.argument!=0{return Err(Error::Argument)}0}};
        if !matches!(h.operation,Operation::OpenTcp|Operation::SendUdp)&&(h.address!=[0;4]||h.port!=0){return Err(Error::Reserved)}
        if matches!(h.operation,Operation::OpenTcp|Operation::SendUdp|Operation::ReceiveUdp|Operation::Retire)&&h.connection!=0{return Err(Error::Reserved)}
        if p.bytes[HEADER+data_length..].iter().any(|b|*b!=0){return Err(Error::Reserved)}Ok(p)
    }
    pub fn decode_response(bytes:&[u8],expected:HeaderFields)->Result<Self,Error>{
        let p=Self::decode(bytes,true)?;let h=p.header();if h.operation!=expected.operation||h.request!=expected.request||h.epoch!=expected.epoch||h.address!=expected.address||h.port!=expected.port||h.operation!=Operation::OpenTcp&&h.connection!=expected.connection{return Err(Error::Binding)}
        let length=if p.status()==Status::Ok&&matches!(h.operation,Operation::ReceiveTcp|Operation::ReceiveUdp){h.argument as usize}else{0};if length>DATA||p.bytes[HEADER+length..].iter().any(|b|*b!=0){return Err(Error::Reserved)}
        if p.status()!=Status::Ok&&h.argument!=0{return Err(Error::Argument)}
        if p.status()==Status::Ok{match h.operation{Operation::OpenTcp=>{if h.connection==0||h.argument!=ConnectionState::Connecting as u32{return Err(Error::Argument)}},Operation::SendTcp|Operation::SendUdp=>{if h.argument>expected.argument{return Err(Error::Argument)}},Operation::State=>{ConnectionState::decode(h.argument)?;},Operation::CloseTcp|Operation::Retire=>if h.argument!=0{return Err(Error::Argument)},_=>{}}}Ok(p)
    }
    /// Correlation only, for a same-peer malformed-frame refusal. The native
    /// service still authenticates the actual sender and ingress beforehand.
    pub fn refusal_header(bytes:&[u8],epoch:u64)->Option<HeaderFields>{
        if bytes.len()<HEADER||&bytes[..4]!=b"NET1"||get16(bytes,4)!=VERSION{return None}
        Some(HeaderFields{operation:Operation::decode(get16(bytes,6)).ok()?,request:get64(bytes,8),epoch,connection:get64(bytes,24),argument:0,address:bytes[40..44].try_into().unwrap(),port:get16(bytes,44)})
    }
    pub fn header(&self)->HeaderFields{HeaderFields{operation:Operation::decode(get16(&self.bytes,6)&!REPLY).unwrap(),request:get64(&self.bytes,8),epoch:get64(&self.bytes,16),connection:get64(&self.bytes,24),argument:get32(&self.bytes,36),address:self.bytes[40..44].try_into().unwrap(),port:get16(&self.bytes,44)}}
    pub fn status(&self)->Status{Status::decode(get32(&self.bytes,32)).unwrap()}
    pub fn bytes(&self)->&[u8;SIZE]{&self.bytes}
    pub fn data(&self)->&[u8]{let h=self.header();let length=if matches!(h.operation,Operation::SendTcp|Operation::SendUdp|Operation::ReceiveTcp|Operation::ReceiveUdp){h.argument as usize}else{0};&self.bytes[HEADER..HEADER+length.min(DATA)]}
}
