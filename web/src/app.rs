use yew::prelude::*;
use crate::components::{
    gui::Gui,
};

#[component]
pub fn App() -> Html {
    html! {
        <main>
            <Gui />
        </main>
    }
}
