use bumpalo::Bump;
use iced_web::{Widget, Element, Bus, style::Sheet};
use dodrio::Node;

pub struct Link<'a, Message> {
    content: Element<'a, Message>,
    href: String,
}

impl<'a, Message> Link<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>, href: &str) -> Self {
        Self { content: content.into(), href: href.to_owned() }
    }
}

impl<'a, Message> Widget<Message> for Link<'a, Message> where Message: 'static + Clone {
    fn node<'b>(&self, bump: &'b Bump, bus: &Bus<Message>, style_sheet: &mut Sheet<'b>) -> Node<'b> {
        dodrio::builder::a(bump)
            .attr(
                "href",
                bumpalo::format!(in bump, "{}", self.href)
                .into_bump_str(),
            )
            .children(vec![self.content.node(bump, bus, style_sheet)])
            .finish()
    }
}

impl<'a, Message> From<Link<'a, Message>> for Element<'a, Message> where Message: 'static + Clone {
    fn from(link: Link<'a, Message>) -> Element<'a, Message> {
        Element::new(link)
    }
}
