//! Real ring3 journal service. Only completed WRITE then completed FLUSH may ACK.
use crate::{journal, rt, wire};
use rt::Message;
use wire::Status;

fn poll(cap:u64, ticket:u64, length:usize) -> Result<Message,Status> {
    for _ in 0..rt::LIMIT {
        let (code,n)=rt::call(0x41,[cap,ticket,rt::OUT,320,0,0]);
        if code==-8 && n==0 {rt::yield_now();continue;}
        if code!=0 {return Err(Status::from_code(code));}
        let m=rt::output();
        if n!=length as u64 || m.sender()!=0 || m.tag()!=ticket || m.length()!=length || m.cap_count()!=0
            || m.bytes[24..64].iter().any(|b|*b!=0) || m.bytes[64+length..].iter().any(|b|*b!=0) {return Err(Status::Protocol);}
        return Ok(m);
    } Err(Status::Timeout)
}
fn submit(number:u64,args:[u64;6]) -> Result<u64,Status> {
    // No replay: even EAGAIN is reported; the caller never silently reissues a PUT.
    let (code,ticket)=rt::call(number,args);
    if code==0 {Ok(ticket)} else {if ticket!=0 {rt::fail(70);}Err(Status::from_code(code))}
}
fn flush(cap:u64) -> Result<(),Status> {let ticket=submit(0x43,[cap,0,0,0,0,0])?;poll(cap,ticket,0)?;Ok(())}
fn read_sector(cap:u64,relative:u64) -> Result<[u8;512],Status> {
    let mut bytes=[0;512];
    for part in 0..4 {
        let ticket=submit(0x40,[cap,relative,(part*128) as u64,128,0,0])?;
        let m=poll(cap,ticket,128)?;bytes[part*128..part*128+128].copy_from_slice(&m.bytes[64..192]);
    } Ok(bytes)
}
fn stage_record(record:&[u8;512]) {
    for (i,b) in record.iter().enumerate() {
        // SAFETY: Exact 512 bytes in our fixed private writable sector page.
        unsafe {(rt::SECTOR as *mut u8).add(i).write_volatile(*b);}
    }
}
fn write(cap:u64,relative:u64,ptr:u64) -> Result<(),Status> {
    let ticket=submit(0x42,[cap,relative,ptr,512,0,0])?;poll(cap,ticket,0)?;Ok(())
}
fn preflight() -> Result<(),Status> {
    let cap=rt::word(200);let read=rt::word(208);let writecap=rt::word(216);let flushcap=rt::word(224);
    for other in [rt::word(56),0] {rt::expect(rt::call(0x42,[other,0,rt::SECTOR,512,0,0]),if other==0 {-4}else{-6});}
    for other in [read,flushcap] {rt::expect(rt::call(0x42,[other,0,rt::SECTOR,512,0,0]),-5);}
    for other in [read,writecap] {rt::expect(rt::call(0x43,[other,0,0,0,0,0]),-5);}
    rt::expect(rt::call(0x42,[cap,1,rt::SECTOR,512,0,0]),-5);
    for len in [0,511,513] {rt::expect(rt::call(0x42,[cap,0,rt::SECTOR,len,0,0]),-11);}
    for ptr in [0,u64::MAX,0x40014f01] {rt::expect(rt::call(0x42,[cap,0,ptr,512,0,0]),-3);}
    // Valid readonly input is readable, never required to be writable.
    write(cap,0,0x40014000)?;flush(cap)?;
    if read_sector(cap,0)? != [0xa7;512] {rt::fail(71);}
    // 255 RW bytes then 257 RO bytes: snapshot must survive user mutation.
    for i in 0..255 {
        // SAFETY: These bytes are the tail of our mapped private sector page.
        unsafe {(0x40013f01 as *mut u8).add(i).write_volatile(0x3c);}
    }
    let ticket=submit(0x42,[cap,0,0x40013f01,512,0,0])?;
    for i in 0..255 {unsafe {(0x40013f01 as *mut u8).add(i).write_volatile(0xe1);}}
    rt::expect(rt::call(0x41,[read,ticket,rt::OUT,320,0,0]),-5);
    rt::expect(rt::call(0x41,[cap,ticket,0,320,0,0]),-3);
    rt::expect(rt::call(0x41,[cap,ticket,0x40014000,320,0,0]),-3);
    poll(cap,ticket,0)?;flush(cap)?;
    let bytes=read_sector(cap,0)?;
    if bytes[..255].iter().any(|b|*b!=0x3c) || bytes[255..].iter().any(|b|*b!=0xa7) {rt::fail(72);}
    rt::put(232,1);Ok(())
}
fn announce(epoch:u64) {
    let mut handles=[0;2];
    for (i,root) in [rt::word(56),rt::word(80)].into_iter().enumerate() {
        let (code,h)=rt::call(0x11,[root,9,0,0,0,0]);if code!=0 {rt::fail(73);}handles[i]=h;
    }
    let mut p=[0;32];wire::put64(&mut p,0,2);wire::put64(&mut p,8,epoch);wire::put64(&mut p,24,2);
    let mut notice=Message::new();notice.payload(wire::ANNOUNCE,&p);notice.add_caps(&handles,9);rt::send(rt::word(72),&notice);
    for h in handles {rt::expect(rt::call(0x10,[h,0,0,0,0,0]),0);}
}
fn startup_error(error:Status) -> ! {rt::put(232,error as i32 as i64 as u64);rt::put(40,1);rt::worker()}
pub fn run() -> ! {
    let epoch=rt::word(32);let cap=rt::word(48);
    if epoch==0 || cap==0 {rt::fail(74);}
    if rt::word(112)==2 {if let Err(e)=preflight() {startup_error(e);}}
    let mut records=[[0;512];4];
    for (i,record) in records.iter_mut().enumerate() {*record=match read_sector(cap,i as u64) {Ok(v)=>v,Err(e)=>startup_error(e)};}
    let mut current=match journal::recover(&records) {Ok(s)=>s,Err(_)=>startup_error(Status::Corrupt)};
    if let Err(e)=flush(cap) {startup_error(e);}
    rt::put(128,current.generation);rt::put(120,wire::checksum_update(wire::FNV_OFFSET,&current.value));
    announce(epoch);rt::put(192,1);
    let mut uncertain=false;
    loop {
        for (ingress,root) in [rt::word(56),rt::word(80)].into_iter().enumerate() {
            let Some(mut message)=rt::receive(root) else {continue;};
            let caps=message.close_caps();if message.sender()!=rt::word(160) {continue;}
            let mut body=[0;wire::SIZE];body.copy_from_slice(&message.bytes[64..256]);let h=wire::header(&body);
            let valid=if caps || message.tag()!=wire::TAG || message.length()!=wire::SIZE {Err(Status::Invalid)}else{wire::validate_request(&body,epoch,ingress==1)};
            let mut status=Status::Ok;let mut value=None;
            match valid {
                Err(e)=>status=e,
                Ok(_) if uncertain=>status=Status::Unknown,
                Ok(v) if v.operation==wire::GET=>value=Some(current.value),
                Ok(v)=>{
                    let mut next=[0;128];next.copy_from_slice(&body[48..176]);
                    match journal::prepare(&current,v.generation,v.request_id,&next) {
                        Err(journal::Error::Stale)=>status=Status::Stale,
                        Err(journal::Error::Full)=>status=Status::Full,
                        Err(_)=>status=Status::Corrupt,
                        Ok(record)=>{
                            stage_record(&record);rt::put(192,2);
                            if write(cap,current.generation,rt::SECTOR).and_then(|_|flush(cap)).is_err() {
                                uncertain=true;status=Status::Unknown;rt::put(232,(-16i64) as u64);
                            }else{
                                current=journal::decode(&record,current.generation as usize).unwrap_or_else(|_|rt::fail(75));
                                rt::put(128,current.generation);rt::put(120,wire::checksum_update(wire::FNV_OFFSET,&current.value));rt::put(192,3);
                            }
                        }
                    }
                }
            }
            let bytes=wire::reply(h,epoch,current.generation,status,value.as_ref());let mut reply=Message::new();reply.payload(wire::TAG,&bytes);rt::send(rt::word(88),&reply);
        }
        rt::yield_now();
    }
}
