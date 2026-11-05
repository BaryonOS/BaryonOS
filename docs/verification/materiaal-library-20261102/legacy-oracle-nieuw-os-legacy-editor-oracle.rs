use nieuw_os_materiaal::{controller::{View,Screen,Focus,Status},render::Frame,workspace::Workspace,Scale,Viewport,TILE_PIXELS};
static WORKSPACE:Workspace=Workspace::new();
fn view(revision:u64)->View<'static>{View{draw_pointer:false,desktop:None,viewport:Viewport::new(768,536,Scale::Percent100,1).unwrap(),generation:1,dirty:true,screen:Screen::Editor,focus:Focus::Document,status:if revision==0{Status::Dirty}else{Status::Saved{revision}},objects:&[],workspace:&WORKSPACE,list_loaded:true,list_scroll:0,contents:Some("nieuw os werkt"),title:b"Nieuw document",revision,caret:14,caret_trailing:false,anchor:14,scroll:0,editable:true,can_save:true,can_inspect:true,saving:false,document_dirty:revision==0,cursor:None}}
fn main(){
 let before=Frame::new(view(0));let after=Frame::new(view(1));let l=before.layout();let mut old_changed=0;let mut editor_changed=0;let(mut left,mut top,mut right,mut bottom)=(u32::MAX,u32::MAX,0,0);
 for rect in before.tiles(){let(mut a,mut b)=([0;TILE_PIXELS],[0;TILE_PIXELS]);before.render_tile(rect.x,rect.y,&mut a).unwrap();after.render_tile(rect.x,rect.y,&mut b).unwrap();for dy in 0..rect.height{for dx in 0..rect.width{let i=(dy*rect.width+dx)as usize;let(x,y)=(rect.x+dx,rect.y+dy);if a[i]!=b[i]{if x>=24&&x<744&&y>=144&&y<472{old_changed+=1;left=left.min(x);top=top.min(y);right=right.max(x);bottom=bottom.max(y);}if l.editor.contains(x,y){editor_changed+=1;}}}}}
 println!("layout_editor={:?}; old144_region_changes={old_changed}; actual_editor_changes={editor_changed}; old_region_changed_bounds={left},{top},{right},{bottom}",l.editor);
}
