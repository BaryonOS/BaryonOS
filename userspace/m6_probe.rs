#![no_std]
#![no_main]
#[path = "../services/block/src/wire.rs"]
#[allow(dead_code)] // Only its existing scalar codecs/checksum are reused.
mod legacy_wire;
// Shared primitive codec only; the M5 protocol remains a separate module.
#[path = "../services/block/src/journal.rs"]
mod journal;
#[path = "../services/block/src/m6_wire.rs"]
mod wire;
#[path = "m6_runtime.rs"]
mod rt;
#[path = "../services/block/src/m6_user_service.rs"]
mod block_service;
use rt::Message;
struct Connection { caps: [u64; 2], epoch: u64, service_pid: u64 }
impl Connection {
    fn new() -> Self { Self { caps: [0; 2], epoch: 0, service_pid: 0 } }
    fn install(&mut self, mut notice: Message) {
        let p = &notice.bytes[64..96];
        let epoch = wire::get64(p, 8); let pid = wire::get64(p, 16);
        let valid = notice.sender() == rt::word(168) && notice.tag() == wire::REBIND
            && notice.length() == 32 && notice.cap_count() == 2
            && wire::get64(p, 0) == 2 && wire::get64(p, 24) == 2
            && epoch == self.epoch + 1 && pid > self.service_pid
            && (self.epoch != 0 || pid == rt::word(176))
            && notice.rights(0) == 1 && notice.rights(1) == 1;
        if !valid { notice.close_caps(); rt::fail(30); }
        let fresh = [notice.cap(0), notice.cap(1)];
        if self.epoch != 0 {
            let payload = wire::request(1, self.epoch, 0, wire::GET, None);
            let mut stale = Message::new(); stale.payload(wire::TAG, &payload);
            rt::expect(rt::send_once(self.caps[0], &stale), -10);
            for handle in self.caps { rt::expect(rt::call(0x10, [handle, 0, 0, 0, 0, 0]), 0); }
        }
        self.caps = fresh; self.epoch = epoch; self.service_pid = pid;
        rt::put(136, epoch);
    }
    fn try_rebind(&mut self) -> bool {
        if let Some(notice) = rt::receive(rt::word(80)) { self.install(notice); true } else { false }
    }
    fn wait_rebind(&mut self) {
        for _ in 0..rt::LIMIT { if self.try_rebind() { return; } rt::yield_now(); }
        rt::fail(31)
    }
}

enum Rpc { Reply(Message), Rebound }
fn rpc(conn: &mut Connection, scope: usize, message: &Message) -> Rpc {
    let mut published = false;
    for _ in 0..rt::LIMIT {
        let sent = rt::send_once(conn.caps[scope], message);
        match sent {
            (0, 0) => { published = true; break; },
            (-8, 0) => { if conn.try_rebind() { return Rpc::Rebound; } rt::yield_now(); },
            (-10, 0) if (18..=23).contains(&rt::word(112)) => { conn.wait_rebind(); return Rpc::Rebound; },
            _ => rt::fail(32),
        }
    }
    if !published { rt::fail(49); }
    rt::put(144, rt::word(144) + 1);
    for _ in 0..rt::LIMIT {
        if conn.try_rebind() { return Rpc::Rebound; }
        if let Some(mut response) = rt::receive(rt::word(56)) {
            let caps = response.close_caps();
            if caps || response.sender() != conn.service_pid || response.tag() != wire::TAG
                || response.length() != wire::SIZE { rt::fail(33); }
            return Rpc::Reply(response);
        }
        rt::yield_now();
    }
    rt::fail(34)
}
fn body(message:&Message)->[u8;192] {let mut b=[0;192];b.copy_from_slice(&message.bytes[64..256]);b}
fn request(conn:&Connection,id:u64,generation:u64,op:u16,value:Option<&[u8;128]>)->Message {
    let mut m=Message::new();m.payload(wire::TAG,&wire::request(id,conn.epoch,generation,op,value));m
}
fn reply(conn:&Connection,req:&Message,response:Message)->(wire::Header,[u8;128]) {
    let bytes=body(&response);let h=wire::validate_reply(&bytes,wire::header(&body(req)),conn.epoch).unwrap_or_else(|_|rt::fail(50));
    let mut value=[0;128];value.copy_from_slice(&bytes[48..176]);(h,value)
}
fn get(conn:&mut Connection)->(u64,[u8;128]) {
    for _ in 0..rt::LIMIT {
        let m=request(conn,1,0,wire::GET,None);
        match rpc(conn,0,&m) {Rpc::Rebound=>continue,Rpc::Reply(r)=>{let(h,v)=reply(conn,&m,r);if h.status!=0 {rt::fail(51);}return(h.generation,v);}}
    }rt::fail(52)
}
fn negative(conn:&mut Connection,scope:usize,m:Message,status:wire::Status) {
    let source=if m.cap_count()!=0 {m.cap(0)}else{0};
    match rpc(conn,scope,&m) {Rpc::Rebound=>rt::fail(53),Rpc::Reply(r)=>{
        if source!=0 {rt::expect(rt::call(0x10,[source,0,0,0,0,0]),0);}
        let(h,_)=reply(conn,&m,r);if h.status!=status as i32 {rt::fail(54);}
    }}rt::put(152,rt::word(152)+1);
}
fn malformed(conn:&mut Connection) {
    for fault in 0..15 {
        let put=matches!(fault,5|6|7|8|14);
        let mut b=wire::request(100+fault,conn.epoch,if put {1}else{0},if put {wire::PUT}else{wire::GET},if put {Some(&[0x55;128])}else{None});
        let status=match fault {
            0=>{b[0]=1;wire::Status::Invalid},1=>{b[2]=3;wire::Status::NoSys},
            2=>{b[4]=1;wire::Status::Invalid},3=>{b[40]=1;wire::Status::Invalid},
            4=>{wire::put32(&mut b,32,1);wire::Status::Invalid},
            5=>{wire::put32(&mut b,32,0);wire::Status::Range},6=>{wire::put32(&mut b,32,129);wire::Status::Range},
            7=>{wire::put64(&mut b,24,0);wire::Status::Range},8=>{wire::put64(&mut b,24,u64::MAX);wire::Status::Range},
            9=>{wire::put64(&mut b,16,0);wire::Status::Stale},10=>{b[48]=1;wire::Status::Invalid},
            14=>wire::Status::Access,_=>wire::Status::Invalid,
        };
        let mut m=Message::new();m.payload(wire::TAG,&b);
        if fault==11 {wire::put32(&mut m.bytes,16,191);}
        if fault==12 {wire::put32(&mut m.bytes,16,193);}
        if fault==13 {let(code,h)=rt::call(0x11,[rt::word(56),9,0,0,0,0]);if code!=0 {rt::fail(55);}m.add_caps(&[h,h],1);}
        negative(conn,if fault==14 {0}else{1},m,status);
    }
}
fn client()->! {
    let case=rt::word(112);let mut conn=Connection::new();conn.wait_rebind();
    let (initial_gen,initial)=get(&mut conn);
    if matches!(case,14|30) {
        rt::put(120,wire::checksum_update(wire::FNV_OFFSET,&initial));rt::put(128,initial_gen);rt::put(232,initial_gen);rt::put(40,1);rt::worker();
    }
    if initial_gen!=1 {rt::fail(56);}
    if case==1 {malformed(&mut conn);}
    if case==16 {let m=request(&conn,99,2,wire::PUT,Some(&initial));negative(&mut conn,1,m,wire::Status::Stale);}
    let mut current=initial;let mut generation=initial_gen;
    let iterations=if case==3 {3}else{1};
    for _ in 0..iterations {
        let mut next=current;for b in &mut next {*b=b.wrapping_add((0x31+generation+1) as u8);}
        let m=request(&conn,generation+1,generation,wire::PUT,Some(&next));rt::put(192,2);
        match rpc(&mut conn,1,&m) {
            Rpc::Rebound=>{
                if !(18..=23).contains(&case) {rt::fail(57);}
                let(g,v)=get(&mut conn);
                if !(g==1&&v==initial || g==2&&v==next) || (case>=22&&g!=2) {rt::fail(58);}
                current=v;generation=g;
            },
            Rpc::Reply(r)=>{
                let(h,_)=reply(&conn,&m,r);
                if matches!(case,4..=8|11..=13) {
                    if h.status!=-16 {rt::fail(59);}rt::put(232,(-16i64) as u64);rt::put(40,1);rt::worker();
                }
                if h.status!=0 || h.generation!=generation+1 {rt::fail(60);}
                current=next;generation+=1;
                if (18..=23).contains(&case) {
                    conn.wait_rebind();let(g,v)=get(&mut conn);if g!=generation||v!=current {rt::fail(61);}
                }
            }
        }
    }
    if case==3 {let m=request(&conn,5,4,wire::PUT,Some(&current));negative(&mut conn,1,m,wire::Status::Full);}
    let(g,value)=get(&mut conn);if g!=generation||value!=current {rt::fail(62);}
    rt::put(120,wire::checksum_update(wire::FNV_OFFSET,&value));rt::put(128,generation);rt::put(232,generation);rt::put(192,5);rt::put(40,1);rt::worker()
}
fn supervisor()->! {
    let mut epoch=0;let mut pid=0;
    loop {
        let Some(mut notice)=rt::receive(rt::word(56)) else {rt::yield_now();continue;};
        let p=&notice.bytes[64..96];let next=wire::get64(p,8);let sender=notice.sender();
        if notice.tag()!=wire::ANNOUNCE || notice.length()!=32 || notice.cap_count()!=2
            || wire::get64(p,0)!=2 || wire::get64(p,16)!=0 || wire::get64(p,24)!=2
            || sender==rt::word(160) || sender==rt::word(168) || sender<=pid
            || (epoch==0 && sender!=rt::word(176)) || next!=epoch+1 || notice.rights(0)!=9 || notice.rights(1)!=9 {
                notice.close_caps();rt::fail(63);
            }
        let handles=[notice.cap(0),notice.cap(1)];let mut payload=[0;32];payload.copy_from_slice(p);wire::put64(&mut payload,16,sender);
        let mut m=Message::new();m.payload(wire::REBIND,&payload);m.add_caps(&handles,1);rt::send(rt::word(64),&m);
        for h in handles {rt::expect(rt::call(0x10,[h,0,0,0,0,0]),0);}
        epoch=next;pid=sender;rt::put(136,epoch);rt::put(40,1);
    }
}
#[no_mangle]
#[link_section=".text.entry"]
pub extern "C" fn _start()->! {
    if rt::word(24)!=0 || rt::word(16)==0 || rt::call(0,[0;6])!=(0,(1<<32)|2) {rt::fail(64);}
    match rt::word(0) {0=>block_service::run(),1=>client(),2=>supervisor(),_=>rt::fail(65)}
}
#[panic_handler]
fn panic(_:&core::panic::PanicInfo)->! {rt::fail(0xff)}
