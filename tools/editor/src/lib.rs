use iced::Widget;

#[derive(Debug)]
pub struct Editor {}

#[derive(Debug, Clone)]
pub enum Message {}

impl Editor {
    pub fn boot() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: Message) {}

    pub fn view(&self) -> impl Widget<Message> {
        iced::widget::space()
    }

    pub fn title(&self) -> String {
        format!("{}@v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}
