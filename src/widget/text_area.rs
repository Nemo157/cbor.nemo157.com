use bumpalo::Bump;
use iced_web::{Bus, Element, Widget, style::Sheet};
use std::rc::Rc;
use dodrio::Node;

pub struct TextArea<Message> {
    placeholder: String,
    value: String,
    on_change: Rc<Box<dyn Fn(String) -> Message>>,
    on_submit: Option<Message>,
}

impl<Message> TextArea<Message> {
    pub fn new(placeholder: &str, value: &str, on_change: impl Fn(String) -> Message + 'static) -> Self {
        Self {
            placeholder: String::from(placeholder),
            value: String::from(value),
            on_change: Rc::new(Box::new(on_change)),
            on_submit: None,
        }
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }
}

impl<Message> Widget<Message> for TextArea<Message>
where
    Message: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b Bump,
        bus: &Bus<Message>,
        _style_sheet: &mut Sheet<'b>,
    ) -> Node<'b> {
        use wasm_bindgen::JsCast;

        let on_change = self.on_change.clone();
        let event_bus = bus.clone();

        let mut node = dodrio::builder::textarea(bump)
            .attr(
                "placeholder",
                bumpalo::format!(in bump, "{}", self.placeholder)
                    .into_bump_str(),
            )
            .attr(
                "value",
                bumpalo::format!(in bump, "{}", self.value).into_bump_str(),
            )
            .on("input", move |root, vdom, event| {
                let text_area = match event.target().and_then(|t| {
                    t.dyn_into::<web_sys::HtmlTextAreaElement>().ok()
                }) {
                    None => return,
                    Some(text_area) => text_area,
                };

                event_bus.publish(on_change(text_area.value()), root);
                vdom.schedule_render();
            });

        if let Some(on_submit) = self.on_submit.clone() {
            let event_bus = bus.clone();
            node = node.on("keydown", move |root, vdom, event| {
                let event = match event.dyn_into::<web_sys::KeyboardEvent>().ok() {
                    None => return,
                    Some(event) => event,
                };

                if event.key_code() == 13 && (event.meta_key() || event.ctrl_key()) {
                    event_bus.publish(on_submit.clone(), root);
                    vdom.schedule_render();
                }
            });
        }

        node.finish()
    }
}

impl<'a, Message> From<TextArea<Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(text_area: TextArea<Message>) -> Element<'a, Message> {
        Element::new(text_area)
    }
}
