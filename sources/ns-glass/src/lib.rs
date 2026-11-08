#![forbid(unsafe_code)]
//! Live compositor materials. No wallpaper/image/background capture is used.
use egui::{Color32,Context,Id,Painter,Rect,Stroke,StrokeKind};
pub use eframe::Frame;
#[derive(Clone,Copy,Debug)]
pub struct Material {pub light:bool,pub tint:[u8;3],pub secondary:[u8;3],pub gradient:bool,pub radius:f32,pub strength:f32}
impl Default for Material {fn default()->Self{Self{light:false,tint:[140,161,192],secondary:[178,146,197],gradient:false,radius:12.0,strength:0.35}}}
fn linear(v:u8)->f32{static TABLE:std::sync::OnceLock<[f32;256]>=std::sync::OnceLock::new();let table=TABLE.get_or_init(||std::array::from_fn(|n|{let v=n as f32/255.0;if v<=0.04045{v/12.92}else{((v+0.055)/1.055).powf(2.4)}}));table.get(usize::from(v)).copied().unwrap_or(0.0)}

pub fn luminance(c:Color32)->f32{0.2126*linear(c.r())+0.7152*linear(c.g())+0.0722*linear(c.b())}
pub fn contrast(a:Color32,b:Color32)->f32{let a=luminance(a);let b=luminance(b);(a.max(b)+0.05)/(a.min(b)+0.05)}
pub fn on_color(bg:Color32)->Color32{if contrast(Color32::WHITE,bg)>=contrast(Color32::BLACK,bg){Color32::WHITE}else{Color32::BLACK}}
pub fn readable_accent(accent:Color32,background:Color32)->Color32{
 if contrast(accent,background)>=4.5{return accent;}
 let target=on_color(background);
 for n in 1..=20{let candidate=accent.lerp_to_gamma(target,n as f32/20.0);if contrast(candidate,background)>=4.5{return candidate;}}
 target
}
pub fn background_for_text(background:Color32,text:Color32)->Color32{
 if contrast(background,text)>=4.5{return background;}
 let target=if luminance(text)>luminance(background){Color32::BLACK}else{Color32::WHITE};
 for n in 1..=20{let bg=background.lerp_to_gamma(target,n as f32/20.0);if contrast(bg,text)>=4.5{return bg;}}target
}

fn native_id()->Id { Id::new("ns-live-backdrop-ready-v2") }
pub fn native_ready(ctx:&Context)->bool {ctx.data(|d|d.get_temp::<bool>(native_id())).unwrap_or(false)}
pub fn apply_native(ctx:&Context,frame:&eframe::Frame,enabled:bool,light:bool) {
 #[cfg(target_os="windows")]
 if let Some(window)=frame.winit_window() {
  let key=Id::new("ns-native-backdrop-state-v2");let state=(enabled,light);
  if ctx.data(|d|d.get_temp::<(bool,bool)>(key))==Some(state){return;}
  if enabled { let _=window_vibrancy::apply_mica(window.as_ref(),Some(!light)); }
  let result=if enabled {window_vibrancy::apply_acrylic(window.as_ref(),Some(if light{(235,237,240,72)}else{(18,20,24,72)}))}else{window_vibrancy::clear_acrylic(window.as_ref())};
  ctx.data_mut(|d|{d.insert_temp(key,state);d.insert_temp(native_id(),enabled&&result.is_ok());});
  if let Err(e)=result {ctx.data_mut(|d|d.insert_temp(Id::new("ns-native-backdrop-error"),e.to_string()));}
 }
 #[cfg(not(target_os="windows"))] let _=(ctx,frame,enabled,light);
}
pub fn background(p:&Painter,r:Rect,m:Material){
 if !native_ready(p.ctx()){p.rect_filled(r,0,if m.light{Color32::from_gray(239)}else{Color32::from_gray(24)});}
}
fn body(ctx:&Context,m:Material)->Color32 {
 let base=if m.light{Color32::from_gray(247)}else{Color32::from_gray(28)};
 let tint=Color32::from_rgb(m.tint[0],m.tint[1],m.tint[2]);let base=base.lerp_to_gamma(tint,0.045);
 // Preserve translucency while keeping body text legible over either backdrop extreme.
 let alpha=if native_ready(ctx){if m.light{160}else{180}}else{255};
 Color32::from_rgba_unmultiplied(base.r(),base.g(),base.b(),alpha)
}
/// A translucent text-bearing panel fill, including full-size welcome/backstage pages.
pub fn text_surface_fill(ctx:&Context,m:Material)->Color32 { body(ctx,m) }
fn shapes(ctx:&Context,r:Rect,m:Material)->egui::Shape {
 egui::Shape::Vec(vec![egui::Shape::rect_filled(r,m.radius,body(ctx,m)),
 egui::Shape::rect_stroke(r.shrink(0.5),m.radius,Stroke::new(0.7,Color32::from_white_alpha(if m.light{55}else{28})),StrokeKind::Inside)])
}
fn should_paint(p:&Painter,r:Rect)->bool {
 if !native_ready(p.ctx()){return true;}
 // Leave the main workspace clear; only the smaller floating controls need a scrim.
 if r.area()>=p.ctx().content_rect().area()*0.60{return false;}
 let frame=p.ctx().cumulative_frame_nr();let key=Id::new(("ns-glass-covered",p.layer_id()));
 p.ctx().data_mut(|d|{
  let mut cache=d.get_temp::<(u64,Vec<Rect>)>(key).unwrap_or((frame,Vec::new()));
  if cache.0!=frame{cache=(frame,Vec::new());}
  if cache.1.iter().any(|parent|parent.contains_rect(r)){return false;}
  if cache.1.len()<256{cache.1.push(r);}
  d.insert_temp(key,cache);true
 })
}
pub fn surface(p:&Painter,r:Rect,m:Material){if r.is_finite()&&r.width()>0.0&&r.height()>0.0&&should_paint(p,r){p.add(shapes(p.ctx(),r,m));}}
pub fn reserve_surface(p:&Painter)->egui::layers::ShapeIdx{p.add(egui::Shape::Noop)}
pub fn finish_surface(p:&Painter,slot:egui::layers::ShapeIdx,r:Rect,m:Material){if r.is_finite()&&r.width()>0.0&&r.height()>0.0&&should_paint(p,r){p.set(slot,shapes(p.ctx(),r,m));}}
pub fn feedback(ui:&egui::Ui,response:&egui::Response,accent:Color32,selected:bool){
 let down=response.is_pointer_button_down_on();let hovered=response.hovered();
 let amount=ui.ctx().animate_value_with_time(response.id.with("ns-pressure"),if down{1.0}else if hovered{0.35}else{0.0},0.11);
 if amount>0.01||selected {let rect=response.rect.shrink(0.5+amount*0.6);
 ui.painter().rect_filled(rect,6.0,accent.gamma_multiply(0.08*amount));
 ui.painter().rect_stroke(rect,6.0,Stroke::new(1.0+amount,accent.gamma_multiply(if selected{1.0}else{0.35+amount*0.5})),StrokeKind::Inside);}
 if response.has_focus(){ui.painter().rect_stroke(response.rect.expand(1.5),7.0,Stroke::new(1.6,accent),StrokeKind::Outside);}
}
/// Expand the existing moon's theme choices leftwards; retain the layer through collapse.
pub fn theme_popup(ui:&mut egui::Ui,response:&egui::Response,draw:impl FnOnce(&mut egui::Ui)->bool) {
 let ctx=ui.ctx().clone();let id=response.id.with("ns-theme-strip");
 let mut open=ctx.data(|d|d.get_temp::<bool>(id)).unwrap_or(false);
 if response.clicked(){open=!open;}
 if ctx.input(|i|i.key_pressed(egui::Key::Escape)){open=false;}
 let progress=ctx.animate_bool_with_time(id.with("motion"),open,0.22);
 if progress>0.001||open {
  let frame=egui::Frame::popup(ui.style()).corner_radius(24).inner_margin(10);
  let mut chosen=false;
  if let Some(out)=egui::Popup::from_response(response).id(id).open(true)
   .align(egui::RectAlign::BOTTOM_END).close_behavior(egui::PopupCloseBehavior::IgnoreClicks).frame(frame)
   .show(|ui|{chosen=draw(ui);}) {
   let scale=0.05+0.95*progress;let anchor=response.rect.right_bottom();
   ctx.set_transform_layer(out.response.layer_id,egui::emath::TSTransform{translation:anchor.to_vec2()*(1.0-scale),scaling:scale});
   if !response.clicked()&&ctx.input(|i|i.pointer.any_pressed()) {
    if let Some(pos)=ctx.pointer_interact_pos(){let rect=ctx.layer_transform_to_global(out.response.layer_id).unwrap_or_default()*out.response.rect;
     if !rect.contains(pos)&&!response.rect.contains(pos){open=false;}}
   }
  }
  if chosen{open=false;}
 }
 ctx.data_mut(|d|d.insert_temp(id,open));
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn translucent_body_is_readable_over_both_extreme_backdrops(){let ctx=Context::default();ctx.data_mut(|d|d.insert_temp(native_id(),true));
 for light in [false,true]{for tint in [[255,255,255],[0,0,0],[255,0,0],[0,255,0],[0,0,255]]{
 let c=body(&ctx,Material{light,tint,..Default::default()}).to_srgba_unmultiplied();assert!(c[3]<255);
 for behind in [0.0,255.0]{let a=c[3]as f32/255.0;let rgb=[0,1,2].map(|i|(c[i]as f32*a+behind*(1.0-a))as u8);
 assert!(contrast(if light{Color32::from_gray(25)}else{Color32::from_gray(243)},Color32::from_rgb(rgb[0],rgb[1],rgb[2]))>=4.5);}}}}
 #[test]fn unsupported_backdrop_is_an_opaque_readable_fallback(){let ctx=Context::default();assert!(!native_ready(&ctx));assert_eq!(body(&ctx,Material::default()).a(),255);}
}
