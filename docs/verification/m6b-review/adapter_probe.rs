#![allow(dead_code)]
#[path = "/workspace/scratch/0d3232a3c004/nieuw-os-native/services/objects/src/wire.rs"] mod wire;
#[path = "/workspace/scratch/0d3232a3c004/nieuw-os-native/services/objects/src/journal.rs"] mod journal;
#[path = "/workspace/scratch/0d3232a3c004/nieuw-os-native/services/objects/src/user_service.rs"] mod object_service;
use std::{cell::RefCell, collections::VecDeque};
#[derive(Clone, Copy, Debug, PartialEq, Eq)] enum Fault { Success, WriteSubmit, WritePoll, FlushSubmit, FlushPoll, BadWriteCompletion, WriteTimeout }
#[derive(Clone, Copy, Debug)] struct Stop;
struct Harness { fault: Fault, ticket: u64, pending: u64, output: rt::Message, requests: VecDeque<rt::Message>, responses: Vec<(rt::Message, [u64;3])>, io: [u64;3], words: [u64;128] }
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
        pub fn close_caps(&mut self)->bool { assert_eq!(self.cap_count(),0); false }
        pub fn payload(&mut self,tag:u64,bytes:&[u8]) { wire::put64(&mut self.bytes,8,tag); wire::put32(&mut self.bytes,16,bytes.len() as u32); self.bytes[64..64+bytes.len()].copy_from_slice(bytes); }
    }
    pub fn word(offset:u64)->u64 { state(|h|h.words[offset as usize/8]) }
    pub fn put(offset:u64,value:u64) { state(|h|h.words[offset as usize/8]=value); }
    pub fn fail(code:u64)->! { panic!("unexpected adapter failure: {code}") }
    pub fn worker()->! { panic!("unexpected startup failure") }
    pub fn yield_now() {}
    pub fn receive(endpoint:u64)->Option<Message> { assert!(matches!(endpoint,2|3)); if endpoint==3 {None} else {state(|h|h.requests.pop_front())} }
    pub fn send(endpoint:u64,message:&Message) { assert_eq!(endpoint,4); let done=state(|h| { h.responses.push((*message,h.io)); h.responses.len()==4 }); if done {std::panic::panic_any(Stop);} }
    pub fn output()->Message { state(|h|h.output) }
    pub fn call(number:u64,args:[u64;6])->(i64,u64) {
        assert_eq!(args[0],1);
        state(|h| {
            if number==0x41 {
                assert_eq!(args[1],h.ticket);
                if h.pending==0x42 {
                    if h.fault==Fault::WritePoll {return(-16,0);}
                    if h.fault==Fault::WriteTimeout {return(-8,0);}
                    if h.fault==Fault::BadWriteCompletion {return(0,1);}
                }
                if h.pending==0x43 && h.io[2]==2 && h.fault==Fault::FlushPoll {return(-16,0);}
                let length=if h.pending==0x40 {128} else {0};
                h.output=Message::new(); wire::put64(&mut h.output.bytes,8,h.ticket); wire::put32(&mut h.output.bytes,16,length);
                return(0,length as u64);
            }
            let index=match number {0x40=>0,0x42=>1,0x43=>2,_=>panic!("unexpected syscall {number}")}; h.io[index]+=1;
            if number==0x42 {
                assert_eq!(args[1],0); assert_eq!(args[2],SECTOR); assert_eq!(args[3],512);
                let sector=unsafe {std::slice::from_raw_parts(SECTOR as *const u8,512)};
                let mut record=[0;512]; record.copy_from_slice(sector); assert!(journal::decode_record(&record,0).unwrap().is_some());
                if h.fault==Fault::WriteSubmit {return(-12,0);}
            }
            if number==0x43 && h.io[2]==2 && h.fault==Fault::FlushSubmit {return(-12,0);}
            h.ticket+=1;h.pending=number;(0,h.ticket)
        })
    }
}
extern "C" { fn mmap(address:*mut core::ffi::c_void,length:usize,protection:i32,flags:i32,fd:i32,offset:i64)->*mut core::ffi::c_void; }
fn message(id:u64,object:u64,op:u16,value:Option<&[u8]>)->rt::Message { let mut m=rt::Message::new();m.payload(wire::TAG,&wire::request(id,1,object,0,op,value));wire::put64(&mut m.bytes,0,10);m }
fn main() {
    // Anonymous process-private mapping solely for the adapter's fixed native scratch page.
    let pointer=unsafe {mmap(rt::SECTOR as *mut _,4096,3,0x100022,-1,0)};
    assert_eq!(pointer,rt::SECTOR as *mut _);
    std::panic::set_hook(Box::new(|_| {}));
    for fault in [Fault::Success,Fault::WriteSubmit,Fault::WritePoll,Fault::FlushSubmit,Fault::FlushPoll,Fault::BadWriteCompletion,Fault::WriteTimeout] {
        let requests=VecDeque::from([message(1,2,wire::PUT,Some(b"forbidden")),message(2,0,wire::PUT,Some(b"abc")),message(3,0,wire::PUT,Some(b"abc")),message(4,0,wire::GET,None)]);
        let mut words=[0;128];for(offset,value) in [(32,1),(48,1),(56,2),(80,3),(88,4),(72,5),(160,10),(168,11)] {words[offset/8]=value;}
        H.with(|h| *h.borrow_mut()=Some(Harness{fault,ticket:0,pending:0,output:rt::Message::new(),requests,responses:vec![],io:[0;3],words}));
        let outcome=std::panic::catch_unwind(object_service::run);assert!(outcome.unwrap_err().is::<Stop>());
        state(|h| {
            assert_eq!(h.responses.len(),4);
            assert_eq!(h.responses[0].1,[128,0,1],"wrong-domain request reached I/O");
            let expected=if fault==Fault::Success {[wire::Status::Access,wire::Status::Ok,wire::Status::Stale,wire::Status::Ok]} else {[wire::Status::Access,wire::Status::Unknown,wire::Status::Unknown,wire::Status::Unknown]};
            for (index,(response,counts)) in h.responses.iter().enumerate() {
                let request=if index==0 {wire::request(1,1,2,0,wire::PUT,Some(b"forbidden"))} else if index<3 {wire::request(index as u64+1,1,0,0,wire::PUT,Some(b"abc"))} else {wire::request(4,1,0,0,wire::GET,None)};
                let mut payload=[0;wire::SIZE];payload.copy_from_slice(&response.bytes[64..64+wire::SIZE]);let header=wire::validate_reply(&payload,wire::header(&request),1).unwrap();assert_eq!(header.status,expected[index] as i32);
                if index>=2 {assert_eq!(*counts,h.responses[1].1,"mutation replay after result");}
                if fault!=Fault::Success {assert_eq!(header.revision,0);assert_eq!(header.length,0);}
                if index==3 && fault==Fault::Success {assert_eq!(header.revision,1);assert_eq!(header.length,3);assert_eq!(&payload[64..67],b"abc");}
            }
            assert_eq!(h.io[1],1);assert_eq!(h.words[128/8],if fault==Fault::Success {1} else {0});
            assert_eq!(h.words[232/8],if fault==Fault::Success {0} else {wire::Status::Unknown as i32 as i64 as u64});
            println!("PASS {fault:?}: replies={}, I/O={:?}, sequence={}",h.responses.len(),h.io,h.words[128/8]);
        });
    }
}
