use yew::{html, Component, Context, Html, NodeRef, Properties};
use web_sys::{HtmlCanvasElement, WheelEvent};

pub enum GuiMsg {
	Redraw,
	#[allow(unused)]
	ZoomOut(i32, i32),
	#[allow(unused)]
	ZoomIn(i32, i32),
}

#[derive(Properties, PartialEq)]
pub struct GuiProps {
}

pub struct Gui {
	canvas: NodeRef,
}

impl Component for Gui {
	type Message = GuiMsg;
	type Properties = GuiProps;

	fn create(ctx: &Context<Self>) -> Self {
		ctx.link().send_message(GuiMsg::Redraw);
		Gui {
			canvas: NodeRef::default(),
		}
	}

	fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			GuiMsg::Redraw => {
				let element: HtmlCanvasElement = self.canvas.cast().unwrap();
				let egui_runner = Box::leak(Box::new(crate::egui::WebHandle::new()));
				wasm_bindgen_futures::spawn_local(egui_runner.start(element));
				false
			},
			_ => true,
		}
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		let onwheel = {
			let link = ctx.link().clone();
			move |event: WheelEvent| {
				let delta = event.delta_y();
				if delta > 0.0 {
					link.send_message(GuiMsg::ZoomOut(event.x(), event.y()));
				} else {
					link.send_message(GuiMsg::ZoomIn(event.x(), event.y()));
				}
				event.prevent_default();
			}
		};

		html!(
        	<div>
            	<canvas style="display: block; width: 100vw; height: 100vh;" ref={self.canvas.clone()} onwheel={onwheel}/>
        	</div>
    	)
	}
}