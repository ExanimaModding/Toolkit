use emtk_editor::Editor;

fn main() -> iced::Result {
    iced::application(Editor::boot, Editor::update, Editor::view)
        .title(Editor::title)
        .run()
}
