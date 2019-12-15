use iced::{Application, Column, Text, Command, Element, Settings, Row, button, Button, text_input, TextInput, Radio, Align, Color, Length, Container};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum ParseType {
    Auto,
    Hex,
    Diag,
}

impl Default for ParseType {
    fn default() -> Self {
        ParseType::Auto
    }
}

#[derive(Debug, Default)]
struct Header {
    cbor_me_link: button::State,
    cbor_io_link: button::State,
    cbor_diag_link: button::State,
    cbor_diag_cli_link: button::State,
    github_link: button::State,
    dark_mode_toggle: button::State,
}

#[derive(Debug, Clone)]
enum HeaderMsg {
}

#[derive(Default, Debug)]
struct Input {
    selected_parse_type: ParseType,
    parse_button: button::State,
    input_text_state: text_input::State,
    input_text: String,
}

#[derive(Debug, Clone)]
enum InputMsg {
    SelectParseType(ParseType),
    ParseClicked,
    TextChange(String),
}

#[derive(Default, Debug)]
struct Output {
    hex: String,
    diag: String,
}

#[derive(Debug, Clone)]
enum OutputMsg {
    Update {
        hex: String,
        diag: String,
    }
}

#[derive(Default, Debug)]
struct Playground {
    header: Header,
    input: Input,
    output: Output,
}

#[derive(Debug, Clone)]
enum PlaygroundMsg {
    Header(HeaderMsg),
    Input(InputMsg),
    Output(OutputMsg),
}

const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

fn text(label: &str) -> Text {
    Text::new(label).size(14)
}

fn mono(label: &str) -> Text {
    // TODO: monospace font
    text(label).width(Length::Shrink)
}

fn link<'a, M>(state: &'a mut button::State, label: Text, _url: &str) -> Button<'a, M> {
    Button::new(state, label.color(BLUE)).padding(0).border_radius(0).width(Length::Shrink)
}

impl Header {
    fn update(&mut self, msg: HeaderMsg) {
        match msg {
        }
    }

    fn view(&mut self) -> impl Into<Element<'_, HeaderMsg>> {
        Row::new()
            .align_items(Align::Center)
            .spacing(40)
            .push(
                Row::new()
                .align_items(Align::Center)
                .width(Length::Shrink)
                .push(text("Inspired by"))
                .push(link(&mut self.cbor_me_link, text("cbor.me"), "https://cbor.me")))
            .push(
                Row::new()
                .width(Length::Shrink)
                .push(text("See"))
                .push(link(&mut self.cbor_io_link, text("cbor.io"), "https://cbor.io"))
                .push(text("for more information on what CBOR is")))
            .push(
                Row::new()
                .width(Length::Shrink)
                .push(text("Built on"))
                .push(link(&mut self.cbor_diag_link, mono("cbor-diag"), "https://crates.io/crates/cbor-diag")))
            .push(
                Row::new()
                .width(Length::Shrink)
                .push(text("New CLI Tool:"))
                .push(link(&mut self.cbor_diag_cli_link, mono("cbor-diag-cli"), "https://crates.io/crates/cbor-diag-cli")))
            .push(
                Row::new()
                .width(Length::Shrink)
                .push(text("Hosted on"))
                .push(link(&mut self.github_link, text("GitHub"), "https://github.com/Nemo157/cbor.nemo157.com")))
            .push(Button::new(&mut self.dark_mode_toggle, text("Toggle dark mode")))
    }
}

fn parse(ty: ParseType, text: &str) -> Result<(String, String), String> {
    match ty {
        ParseType::Auto => cbor_diag::parse_hex(text)
            .or_else(|_| cbor_diag::parse_diag(text))
            .map(|v| (v.to_hex(), v.to_diag_pretty()))
            .map_err(|e| e.to_string()),

        ParseType::Hex => cbor_diag::parse_hex(text)
            .map(|v| (v.to_hex(), v.to_diag_pretty()))
            .map_err(|e| e.to_string()),

        ParseType::Diag => cbor_diag::parse_diag(text)
            .map(|v| (v.to_hex(), v.to_diag_pretty()))
            .map_err(|e| e.to_string()),
    }
}


impl Input {
    fn update(&mut self, msg: InputMsg) -> Option<OutputMsg> {
        match msg {
            InputMsg::SelectParseType(ty) => {
                self.selected_parse_type = ty;
                None
            }
            InputMsg::ParseClicked => {
                match parse(self.selected_parse_type, &self.input_text) {
                    Ok((hex, diag)) => Some(OutputMsg::Update { hex, diag }),
                    Err(msg) => Some(OutputMsg::Update { hex: msg, diag: "".to_owned() }),
                }
            }
            InputMsg::TextChange(text) => {
                self.input_text = text;
                None
            }
        }
    }

    fn view(&mut self) -> impl Into<Element<'_, InputMsg>> {
        Column::new()
            .push(
        Row::new()
            .push(Radio::new(ParseType::Auto, "Auto", Some(self.selected_parse_type), InputMsg::SelectParseType))
            .push(Radio::new(ParseType::Hex, "Hex", Some(self.selected_parse_type), InputMsg::SelectParseType))
            .push(Radio::new(ParseType::Diag, "Diagnostic Notation", Some(self.selected_parse_type), InputMsg::SelectParseType))
            .push(Button::new(&mut self.parse_button, text("Parse")).on_press(InputMsg::ParseClicked))
            )
            .push(TextInput::new(&mut self.input_text_state, "value to parse", &self.input_text, InputMsg::TextChange)).height(Length::Fill)
            .height(Length::Fill)
    }
}

impl Output {
    fn update(&mut self, msg: OutputMsg) {
        match msg {
            OutputMsg::Update { hex, diag } => {
                self.hex = hex;
                self.diag = diag;
            }
        }
    }

    fn view(&mut self) -> impl Into<Element<'_, OutputMsg>> {
        Column::new()
            .push(Container::new(Text::new(&self.hex).height(Length::Fill)).height(Length::Fill))
            .push(Container::new(Text::new(&self.diag).height(Length::Fill)))
            .height(Length::Fill)
    }
}

impl Application for Playground {
    type Message = PlaygroundMsg;

    fn new() -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "CBOR Playground".to_owned()
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            PlaygroundMsg::Input(msg) => {
                if let Some(msg) = self.input.update(msg) {
                    self.output.update(msg);
                }
            }
            PlaygroundMsg::Output(msg) => self.output.update(msg),
            PlaygroundMsg::Header(msg) => self.header.update(msg),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new()
            .push(self.header.view().into().map(PlaygroundMsg::Header))
            .push(Row::new()
            .push(self.input.view().into().map(PlaygroundMsg::Input))
            .push(self.output.view().into().map(PlaygroundMsg::Output))
            .height(Length::Fill)
            )
            .height(Length::Fill)
            .into()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    Playground::run(Settings::default())
}
