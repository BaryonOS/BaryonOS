#![allow(dead_code)]
#[path = "../../../services/catalog/src/wire.rs"] mod wire;
#[path = "../../../services/catalog/src/journal.rs"] mod journal;
#[path = "../../../services/catalog/src/user_service.rs"] mod catalog_service;
use std::{cell::RefCell, collections::VecDeque};
#[derive(Clone, Copy, Debug, PartialEq, Eq)] enum Fault { Success, WriteSubmit, WritePoll, FlushSubmit, FlushPoll, BadWriteAmount, BadCompletionSender, BadCompletionTicket, BadCompletionPadding, WriteTimeout, FlushTimeout, Legacy, Corrupt, RenameChangedContent }
#[derive(Clone, Copy, Debug)] struct Stop;
#[derive(Clone, Copy, Debug)] struct StartupStop;
struct Harness { fault: Fault, ticket: u64, pending: u64, output: rt::Message, requests: VecDeque<(u64,rt::Message)>, responses: Vec<(u64,rt::Message,[u64;3])>, io: [u64;3], words: [u64;128], disk: [[u8;512];32], read: (usize,usize), written: Vec<[u8;512]>, closed: usize }
thread_local! { static H: RefCell<Option<Harness>> = const { RefCell::new(None) }; }
fn state<T>(f: impl FnOnce(&mut Harness)->T)->T { H.with(|h| f(h.borrow_mut().as_mut().unwrap())) }
mod rt {
    use super::*;
    pub const LIMIT: usize = 3;
    pub const SECTOR: u64 = 0x40013000;
    pub const OUT: u64 = 0x40012000;
    #[derive(Clone,Copy)] pub struct Message { pub bytes: [u8;320] }
    impl Message {
        pub fn new()->Self { Self { bytes:[0;320] } }
        pub fn sender(&self)->u64 { wire::get64(&self.bytes,0) }
        pub fn tag(&self)->u64 { wire::get64(&self.bytes,8) }
        pub fn length(&self)->usize { wire::get32(&self.bytes,16) as usize }
        pub fn cap_count(&self)->usize { wire::get32(&self.bytes,20) as usize }
        pub fn close_caps(&mut self)->bool { let n=self.cap_count(); state(|h|h.closed+=n); wire::put32(&mut self.bytes,20,0); n!=0 }
        pub fn payload(&mut self,tag:u64,bytes:&[u8]) { assert!(bytes.len()<=256);wire::put64(&mut self.bytes,8,tag);wire::put32(&mut self.bytes,16,bytes.len() as u32);self.bytes[64..64+bytes.len()].copy_from_slice(bytes); }
    }
    pub fn word(offset:u64)->u64 { state(|h|h.words[offset as usize/8]) }
    pub fn put(offset:u64,value:u64) { state(|h|h.words[offset as usize/8]=value); }
    pub fn fail(code:u64)->! { panic!("unexpected adapter failure: {code}") }
    pub fn worker()->! { std::panic::panic_any(StartupStop) }
    pub fn yield_now() {}
    pub fn receive(endpoint:u64)->Option<Message> { assert!(matches!(endpoint,2|3));state(|h| if h.requests.front().is_some_and(|(ep,_)|*ep==endpoint){Some(h.requests.pop_front().unwrap().1)} else {None}) }
    pub fn send(endpoint:u64,message:&Message) { assert!(matches!(endpoint,4|5));let done=state(|h|{h.responses.push((endpoint,*message,h.io));h.requests.is_empty()});if done{std::panic::panic_any(Stop);} }
    pub fn output()->Message { state(|h|h.output) }
    pub fn call(number:u64,args:[u64;6])->(i64,u64) {
        assert_eq!(args[0],1);
        state(|h| {
            if number==0x41 {
                assert_eq!(args[1],h.ticket);
                if h.pending==0x42 {
                    if h.fault==Fault::WritePoll{return(-16,0);}
                    if h.fault==Fault::WriteTimeout{return(-8,0);}
                    if h.fault==Fault::BadWriteAmount{return(0,1);}
                }
                if h.pending==0x43 && h.io[2]==2 {
                    if h.fault==Fault::FlushPoll{return(-16,0);}
                    if h.fault==Fault::FlushTimeout{return(-8,0);}
                }
                let length=if h.pending==0x40{128}else{0};
                h.output=Message::new();wire::put64(&mut h.output.bytes,8,h.ticket);wire::put32(&mut h.output.bytes,16,length);
                if h.pending==0x40 {let(index,start)=h.read;h.output.bytes[64..192].copy_from_slice(&h.disk[index][start..start+128]);}
                if h.pending==0x42 {
                    if h.fault==Fault::BadCompletionSender{wire::put64(&mut h.output.bytes,0,10);}
                    if h.fault==Fault::BadCompletionTicket{wire::put64(&mut h.output.bytes,8,h.ticket+1);}
                    if h.fault==Fault::BadCompletionPadding{h.output.bytes[24]=1;}
                }
                return(0,length as u64);
            }
            let index=match number{0x40=>0,0x42=>1,0x43=>2,_=>panic!("unexpected syscall {number}")};h.io[index]+=1;
            if number==0x40 {h.read=(args[1] as usize,args[2] as usize);assert_eq!(args[3],128);}
            if number==0x42 {
                assert_eq!(args[1],2);assert_eq!(args[2],SECTOR);assert_eq!(args[3],512);
                let sector=unsafe{std::slice::from_raw_parts(SECTOR as *const u8,512)};
                let mut record=[0;512];record.copy_from_slice(sector);
                assert_eq!(&record[..8],b"NOSM6C02");assert_eq!(wire::get64(&record,16),3);assert_eq!(wire::get64(&record,32),0);assert_eq!(wire::get64(&record,40),2);assert_eq!(u16::from_le_bytes([record[48],record[49]]),4);
                assert_eq!(&record[64..67],b"abc");assert!(record[67..192].iter().all(|b|*b==0));assert_eq!(&record[192..199],b"renamed");assert!(record[199..504].iter().all(|b|*b==0));
                assert_eq!(wire::get64(&record,504),checksum(&record));h.written.push(record);
                if h.fault==Fault::WriteSubmit{return(-12,0);}
            }
            if number==0x43 && h.io[2]==2 && h.fault==Fault::FlushSubmit{return(-12,0);}
            h.ticket+=1;h.pending=number;(0,h.ticket)
        })
    }
}
unsafe extern "C"{fn mmap(address:*mut core::ffi::c_void,length:usize,protection:i32,flags:i32,fd:i32,offset:i64)->*mut core::ffi::c_void;}
fn checksum(record:&[u8;512])->u64 {record[..504].iter().chain([0;8].iter()).fold(0xcbf29ce484222325,|sum,b|(sum^u64::from(*b)).wrapping_mul(0x100000001b3))}
fn fixture_record(seq:u64,object:u64,revision:u64,op:u16,name:&[u8],data:&[u8])->[u8;512]{
    let mut r=[0;512];r[..8].copy_from_slice(b"NOSM6C02");r[8..10].copy_from_slice(&2u16.to_le_bytes());r[10..12].copy_from_slice(&64u16.to_le_bytes());
    for(at,v)in[(16,seq),(24,seq+100),(32,object),(40,revision)]{wire::put64(&mut r,at,v);}
    r[48..50].copy_from_slice(&op.to_le_bytes());r[50..52].copy_from_slice(&(data.len() as u16).to_le_bytes());r[52..54].copy_from_slice(&(name.len() as u16).to_le_bytes());r[64..64+data.len()].copy_from_slice(data);r[192..192+name.len()].copy_from_slice(name);let sum=checksum(&r);wire::put64(&mut r,504,sum);r
}
fn message(id:u64,object:u64,revision:u64,op:u16,name:&[u8],data:&[u8],sender:u64)->rt::Message{
    let mut b=[0;224];b[..2].copy_from_slice(&4u16.to_le_bytes());b[2..4].copy_from_slice(&op.to_le_bytes());for(at,v)in[(8,id),(16,1),(24,object),(32,revision)]{wire::put64(&mut b,at,v);}wire::put32(&mut b,40,data.len() as u32);wire::put32(&mut b,48,name.len() as u32);b[64..64+data.len()].copy_from_slice(data);b[192..192+name.len()].copy_from_slice(name);
    let mut m=rt::Message::new();m.payload(0x4d36434c4b000004,&b);wire::put64(&mut m.bytes,0,sender);m
}
fn body(m:&rt::Message)->&[u8]{&m.bytes[64..288]}
fn verify_list(b:&[u8],object:u64,revision:u64,name:&[u8],length:u32){
    assert_eq!(wire::get64(b,32),revision);assert_eq!(wire::get32(b,40),56);assert_eq!(wire::get32(b,48),0);assert_eq!(wire::get32(b,52),1);assert_eq!(wire::get64(b,64),object);assert_eq!(wire::get64(b,72),revision);assert_eq!(wire::get32(b,80),length);assert_eq!(wire::get32(b,84),name.len()as u32);assert_eq!(&b[88..88+name.len()],name);assert!(b[88+name.len()..].iter().all(|x|*x==0));
}

fn authority_probe(){
    let mut spoof=message(1,0,0,5,b"",b"",99);wire::put32(&mut spoof.bytes,20,2);
    let mut cap=message(2,0,0,5,b"",b"",10);wire::put32(&mut cap.bytes,20,1);
    let mut tag=message(5,0,0,5,b"",b"",10);wire::put64(&mut tag.bytes,8,0);
    let mut len=message(6,0,0,5,b"",b"",10);wire::put32(&mut len.bytes,16,223);
    let requests=VecDeque::from([(2,spoof),(2,cap),(3,message(3,2,0,5,b"",b"",10)),(2,message(4,2,0,5,b"",b"",10)),(2,tag),(2,len),(2,message(7,1,2,4,b"resurrect",b"",10)),(2,message(8,1,0,1,b"",b"",10)),(2,message(9,0,0,5,b"",b"",10))]);
    let mut words=[0;128];for(offset,value)in[(32,1),(48,1),(56,2),(80,3),(88,4),(72,5),(160,10),(168,11)]{words[offset/8]=value;}
    let mut disk=[[0;512];32];disk[0]=fixture_record(1,0,1,2,b"original",b"abc");disk[1]=fixture_record(2,2,1,2,b"domain-B-secret",b"secret-data");disk[2]=fixture_record(3,1,1,2,b"deleted-secret",b"deleted-content");disk[3]=fixture_record(4,1,2,3,b"",b"");
    H.with(|h|*h.borrow_mut()=Some(Harness{fault:Fault::Success,ticket:0,pending:0,output:rt::Message::new(),requests,responses:vec![],io:[0;3],words,disk,read:(0,0),written:vec![],closed:0}));
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let outcome=std::panic::catch_unwind(catalog_service::run);std::panic::set_hook(hook);assert!(outcome.unwrap_err().is::<Stop>());
    state(|h|{
        assert_eq!(h.responses.len(),7);assert_eq!(h.closed,3);assert_eq!(h.io,[128,0,1]);assert_eq!(h.words[128/8],4);
        for(i,(_,response,counts))in h.responses.iter().enumerate(){
            assert_eq!(*counts,[128,0,1]);let b=body(response);assert_eq!(wire::get64(b,8),[2,4,5,6,7,8,9][i]);assert_eq!(wire::get32(b,44)as i32,[-1,-5,-1,-1,-19,-19,0][i]);
            if i<6{assert_eq!(wire::get64(b,32),if i<4{0}else{2});assert_eq!(wire::get32(b,40),0);assert!(b[48..].iter().all(|x|*x==0));}
            else{assert_eq!(wire::get64(b,32),3);assert_eq!(wire::get32(b,52),1);assert_eq!(wire::get32(b,40),56);assert_eq!(wire::get64(b,64),0);assert_eq!(wire::get64(b,72),1);assert_eq!(&b[88..96],b"original");assert!(b[96..].iter().all(|x|*x==0));}
        }
        println!("PASS AuthorityAndTombstones: spoofed PID/endpoint denied, 3 attached handles closed, 7 canonical replies, tombstone revision retained, no foreign/deleted name, no writes");
    });
}

fn main(){
    let pointer=unsafe{mmap(rt::SECTOR as *mut _,4096,3,0x100022,-1,0)};assert_eq!(pointer,rt::SECTOR as *mut _);
    for fault in[Fault::Success,Fault::WriteSubmit,Fault::WritePoll,Fault::FlushSubmit,Fault::FlushPoll,Fault::BadWriteAmount,Fault::BadCompletionSender,Fault::BadCompletionTicket,Fault::BadCompletionPadding,Fault::WriteTimeout,Fault::FlushTimeout,Fault::Legacy,Fault::Corrupt,Fault::RenameChangedContent]{
        let requests=VecDeque::from([
          (2,message(1,0,0,5,b"",b"",10)),
          (2,message(2,2,1,4,b"forbidden",b"",10)),
          (2,message(3,0,0,4,b"stale",b"",10)),
          (2,message(4,0,1,4,b"renamed",b"",10)),
          (2,message(5,0,0,5,b"",b"",10)),
          (2,message(6,0,0,1,b"",b"",10)),
          (2,message(7,0,1,4,b"replayed",b"",10)),
          (3,message(8,2,0,5,b"",b"",11))]);
        let mut words=[0;128];for(offset,value)in[(32,1),(48,1),(56,2),(80,3),(88,4),(72,5),(160,10),(168,11)]{words[offset/8]=value;}
        let mut disk=[[0;512];32];disk[0]=fixture_record(1,0,1,2,b"original",b"abc");disk[1]=fixture_record(2,2,1,2,b"domain-B-secret",b"secret-data");
        if fault==Fault::Legacy{disk[0]=[0;512];disk[0][..8].copy_from_slice(b"NOSM6B01");disk[0][8..10].copy_from_slice(&1u16.to_le_bytes());disk[0][10..12].copy_from_slice(&64u16.to_le_bytes());for(at,v)in[(16,1),(24,1),(32,0),(40,1)]{wire::put64(&mut disk[0],at,v);}disk[0][48]=2;let sum=checksum(&disk[0]);wire::put64(&mut disk[0],504,sum);}
        if fault==Fault::Corrupt{disk[1][100]^=0x40;}
        if fault==Fault::RenameChangedContent{disk[2]=fixture_record(3,0,2,4,b"renamed",b"different");}
        H.with(|h|*h.borrow_mut()=Some(Harness{fault,ticket:0,pending:0,output:rt::Message::new(),requests,responses:vec![],io:[0;3],words,disk,read:(0,0),written:vec![],closed:0}));
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let outcome=std::panic::catch_unwind(catalog_service::run);std::panic::set_hook(hook);
        let startup=matches!(fault,Fault::Legacy|Fault::Corrupt|Fault::RenameChangedContent);
        if startup{assert!(outcome.unwrap_err().is::<StartupStop>());state(|h|{assert_eq!(h.responses.len(),0);assert_eq!(h.io[1],0);assert_eq!(h.io[2],0);assert_eq!(h.words[232/8] as i64,-18);assert_ne!(h.words[192/8],1);println!("PASS {fault:?}: refused before READY, reads={}, writes=0, flush=0",h.io[0]);});continue;}
        assert!(outcome.unwrap_err().is::<Stop>());
        state(|h|{
            assert_eq!(h.responses.len(),8);assert_eq!(h.responses[0].2,[128,0,1]);assert_eq!(h.responses[1].2,[128,0,1]);assert_eq!(h.responses[2].2,[128,0,1]);
            let ok=fault==Fault::Success;let expected=if ok{[0,-5,-10,0,0,0,-10,0]}else{[0,-5,-10,-16,-16,-16,-16,-16]};
            for(i,(endpoint,response,counts))in h.responses.iter().enumerate(){
                assert_eq!(*endpoint,if i==7{5}else{4});assert_eq!(response.tag(),0x4d36434c4b000004);assert_eq!(response.length(),224);assert_eq!(response.cap_count(),0);assert!(response.bytes[288..].iter().all(|x|*x==0));
                let b=body(response);assert_eq!(&b[..2],&4u16.to_le_bytes());assert_eq!(wire::get64(b,8),i as u64+1);assert_eq!(wire::get64(b,16),1);assert_eq!(wire::get32(b,44)as i32,expected[i]);assert!(b[4..8].iter().chain(b[56..64].iter()).all(|x|*x==0));
                if expected[i]!=0{assert_eq!(wire::get64(b,32),0);assert_eq!(wire::get32(b,40),0);assert!(b[48..].iter().all(|x|*x==0));}
                if i>=4{assert_eq!(*counts,h.responses[3].2,"I/O after outcome");}
                if i==0{verify_list(b,0,1,b"original",3);}
                if ok && i==4{verify_list(b,0,2,b"renamed",3);}
                if ok && i==7{verify_list(b,2,1,b"domain-B-secret",11);}
                if ok && i==3{assert_eq!(wire::get64(b,32),2);assert!(b[48..].iter().all(|x|*x==0));}
                if ok && i==5{assert_eq!(wire::get64(b,32),2);assert_eq!(wire::get32(b,40),3);assert_eq!(wire::get32(b,48),7);assert_eq!(wire::get32(b,52),0);assert_eq!(&b[64..67],b"abc");assert!(b[67..192].iter().all(|x|*x==0));assert_eq!(&b[192..199],b"renamed");assert!(b[199..].iter().all(|x|*x==0));}
            }
            assert_eq!(h.io[1],1);assert_eq!(h.words[128/8],if ok{3}else{2});assert_eq!(h.written.len(),1);assert_eq!(h.words[232/8]as i64,if ok{0}else{-16});println!("PASS {fault:?}: 8 canonical replies, I/O={:?}, sequence={}, own-domain catalog, rename content unchanged",h.io,h.words[128/8]);
        });
    }
    authority_probe();
}
