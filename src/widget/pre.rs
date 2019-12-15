use bumpalo::Bump;
use dodrio::Node;
use iced_web::{style::Sheet, Bus, Element, Widget};

pub struct Pre {
    content: String,
}

impl Pre {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<'a, Message> Widget<Message> for Pre
where
    Message: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b Bump,
        _bus: &Bus<Message>,
        _style_sheet: &mut Sheet<'b>,
    ) -> Node<'b> {
        let content = bumpalo::format!(in bump, "{}", self.content);

        dodrio::builder::pre(bump)
            .children(vec![dodrio::builder::text(content.into_bump_str())])
            .finish()
    }
}

impl<'a, Message> From<Pre> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(pre: Pre) -> Element<'a, Message> {
        Element::new(pre)
    }
}
