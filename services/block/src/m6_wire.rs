//! Version2 bounded register protocol; no device calls or pointer access.
pub use crate::legacy_wire::{checksum_update, get32, get64, put32, put64, FNV_OFFSET};
pub const SIZE: usize = 192;
pub const VERSION: u16 = 2;
pub const GET: u16 = 1;
pub const PUT: u16 = 2;
pub const TAG: u64 = 0x4d36_424c_4b00_0002;
pub const ANNOUNCE: u64 = 0x4d36_414e_4e00_0002;
pub const REBIND: u64 = 0x4d36_4249_4e00_0002;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status { Ok=0, Invalid=-1, NoSys=-2, Range=-3, Access=-5, Stale=-10, Io=-12, Timeout=-13, NoDevice=-14, Protocol=-15, Unknown=-16, Full=-17, Corrupt=-18 }
impl Status {
    pub fn from_code(code: i64) -> Self {
        match code { 0=>Self::Ok,-1=>Self::Invalid,-2=>Self::NoSys,-3=>Self::Range,-5=>Self::Access,-10=>Self::Stale,-12=>Self::Io,-13=>Self::Timeout,-14=>Self::NoDevice,-16=>Self::Unknown,-17=>Self::Full,-18=>Self::Corrupt,_=>Self::Protocol }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header { pub operation: u16, pub request_id: u64, pub epoch: u64, pub generation: u64, pub length: u32, pub status: i32 }
pub fn header(b: &[u8; SIZE]) -> Header { Header { operation: u16::from_le_bytes([b[2],b[3]]), request_id:get64(b,8),epoch:get64(b,16),generation:get64(b,24),length:get32(b,32),status:get32(b,36) as i32 } }
fn common(b: &[u8; SIZE]) -> bool { b[..2] == [2,0] && get32(b,4)==0 && get64(b,40)==0 && b[176..].iter().all(|v| *v==0) }
pub fn request(id:u64, epoch:u64, generation:u64, op:u16, value:Option<&[u8;128]>) -> [u8; SIZE] {
    let mut b=[0;SIZE]; b[0]=VERSION as u8; b[2..4].copy_from_slice(&op.to_le_bytes()); put64(&mut b,8,id); put64(&mut b,16,epoch); put64(&mut b,24,generation);
    if let Some(v)=value { put32(&mut b,32,128); b[48..176].copy_from_slice(v); } b
}
pub fn validate_request(b:&[u8;SIZE], epoch:u64, allow_put:bool) -> Result<Header,Status> {
    let h=header(b);
    if !common(b) || h.request_id==0 || h.status!=0 { return Err(Status::Invalid); }
    if h.epoch!=epoch { return Err(Status::Stale); }
    match h.operation {
        GET => { if h.generation!=0 || h.length!=0 || b[48..176].iter().any(|v|*v!=0) { return Err(Status::Invalid); } },
        PUT => { if !allow_put { return Err(Status::Access); } if h.length!=128 || !(1..=4).contains(&h.generation) { return Err(Status::Range); } },
        _ => return Err(Status::NoSys),
    } Ok(h)
}
pub fn reply(h:Header, epoch:u64, generation:u64, status:Status, value:Option<&[u8;128]>) -> [u8;SIZE] {
    let mut b=request(h.request_id,epoch,if status==Status::Ok {generation} else {0},h.operation,if status==Status::Ok {value} else {None});
    put32(&mut b,36,status as i32 as u32); b
}
pub fn validate_reply(b:&[u8;SIZE], req:Header, epoch:u64) -> Result<Header,Status> {
    let h=header(b);
    if !common(b) || h.request_id!=req.request_id || h.operation!=req.operation || h.epoch!=epoch { return Err(Status::Invalid); }
    if h.status==0 {
        if !(1..=4).contains(&h.generation) || h.length != if h.operation==GET {128} else {0} { return Err(Status::Invalid); }
    } else if h.length!=0 || h.generation!=0 || !matches!(h.status,-1|-2|-3|-5|-10|-12|-13|-14|-15|-16|-17|-18) { return Err(Status::Invalid); }
    if b[48+h.length as usize..176].iter().any(|v|*v!=0) { return Err(Status::Invalid); } Ok(h)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn get_put_rights_and_zero_request_data() {
        let get=request(1,3,0,GET,None); assert!(validate_request(&get,3,false).is_ok());
        let put=request(2,3,1,PUT,Some(&[4;128])); assert!(validate_request(&put,3,true).is_ok()); assert_eq!(validate_request(&put,3,false),Err(Status::Access));
        let mut bad=get; bad[48]=1; assert_eq!(validate_request(&bad,3,true),Err(Status::Invalid));
    }
    #[test] fn malformed_fields_and_epoch_never_validate() {
        let get=request(1,3,0,GET,None);
        for at in [0,1,4,7,36,40,47,176,191] { let mut b=get;b[at]^=4;assert_eq!(validate_request(&b,3,true),Err(Status::Invalid)); }
        assert_eq!(validate_request(&get,4,true),Err(Status::Stale));
        assert_eq!(validate_request(&request(1,3,0,77,None),3,true),Err(Status::NoSys));
    }
    #[test] fn put_length_generation_bounds() {
        for gen in [0,5,u64::MAX] { assert_eq!(validate_request(&request(2,1,gen,PUT,Some(&[2;128])),1,true),Err(Status::Range)); }
        for len in [0,127,129,u32::MAX] {let mut b=request(2,1,1,PUT,Some(&[2;128]));put32(&mut b,32,len);assert_eq!(validate_request(&b,1,true),Err(Status::Range));}
    }
    #[test] fn reply_ack_and_errors_are_zero_and_correlated() {
        let h=header(&request(2,1,1,PUT,Some(&[2;128])));
        let ack=reply(h,1,2,Status::Ok,None);assert_eq!(validate_reply(&ack,h,1).unwrap().generation,2);assert!(ack[48..].iter().all(|v|*v==0));
        for status in [Status::Unknown,Status::Full,Status::Corrupt] {let b=reply(h,1,2,status,Some(&[4;128]));assert_eq!(validate_reply(&b,h,1).unwrap().status,status as i32);assert!(b[48..].iter().all(|v|*v==0));}
        for at in [8,16,2,40,48,176] {let mut b=ack;b[at]^=1;assert!(validate_reply(&b,h,1).is_err());}
    }
}
