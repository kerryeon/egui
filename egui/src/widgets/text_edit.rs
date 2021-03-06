use crate::{paint::*, *};

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct State {
    /// Character based, NOT bytes.
    /// TODO: store as line + row
    pub cursor: Option<usize>,
}

/// A text region that the user can edit the contents of.
///
/// Example:
///
/// ```
/// # let mut ui = egui::Ui::__test();
/// # let mut my_string = String::new();
/// let response = ui.add(egui::TextEdit::singleline(&mut my_string));
/// if response.lost_kb_focus {
///     // use my_string
/// }
/// ```
#[derive(Debug)]
pub struct TextEdit<'t> {
    text: &'t mut String,
    id: Option<Id>,
    id_source: Option<Id>,
    text_style: Option<TextStyle>,
    text_color: Option<Srgba>,
    multiline: bool,
    enabled: bool,
    desired_width: Option<f32>,
    desired_height_rows: usize,
}

impl<'t> TextEdit<'t> {
    #[deprecated = "Use `TextEdit::singleline` or `TextEdit::multiline` (or the helper `ui.text_edit_singleline`, `ui.text_edit_multiline`) instead"]
    pub fn new(text: &'t mut String) -> Self {
        Self::multiline(text)
    }

    /// Now newlines (`\n`) allowed. Pressing enter key will result in the `TextEdit` loosing focus (`response.lost_kb_focus`).
    pub fn singleline(text: &'t mut String) -> Self {
        TextEdit {
            text,
            id: None,
            id_source: None,
            text_style: None,
            text_color: None,
            multiline: false,
            enabled: true,
            desired_width: None,
            desired_height_rows: 1,
        }
    }

    /// A `TextEdit` for multiple lines. Pressing enter key will create a new line.
    pub fn multiline(text: &'t mut String) -> Self {
        TextEdit {
            text,
            id: None,
            id_source: None,
            text_style: None,
            text_color: None,
            multiline: true,
            enabled: true,
            desired_width: None,
            desired_height_rows: 4,
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    pub fn text_style(mut self, text_style: TextStyle) -> Self {
        self.text_style = Some(text_style);
        self
    }

    pub fn text_color(mut self, text_color: Srgba) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn text_color_opt(mut self, text_color: Option<Srgba>) -> Self {
        self.text_color = text_color;
        self
    }

    /// Default is `true`. If set to `false` then you cannot edit the text.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set to 0.0 to keep as small as possible
    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }

    /// Set the number of rows to show by default.
    /// The default for singleline text is `1`.
    /// The default for multiline text is `4`.
    pub fn desired_rows(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }
}

impl<'t> Widget for TextEdit<'t> {
    fn ui(self, ui: &mut Ui) -> Response {
        let TextEdit {
            text,
            id,
            id_source,
            text_style,
            text_color,
            multiline,
            enabled,
            desired_width,
            desired_height_rows,
        } = self;

        let id = id.unwrap_or_else(|| {
            if let Some(id_source) = id_source {
                ui.make_persistent_id(id_source)
            } else {
                // Since we are only storing cursor, perfect persistence Id not super important
                ui.make_position_id()
            }
        });

        let mut state = ui.memory().text_edit.get(&id).cloned().unwrap_or_default();

        let text_style = text_style.unwrap_or_else(|| ui.style().body_text_style);
        let font = &ui.fonts()[text_style];
        let line_spacing = font.line_spacing();
        let available_width = ui.available().width();
        let mut galley = if multiline {
            font.layout_multiline(text.clone(), available_width)
        } else {
            font.layout_single_line(text.clone())
        };

        let desired_width = desired_width.unwrap_or_else(|| ui.style().spacing.text_edit_width);
        let desired_height = (desired_height_rows.at_least(1) as f32) * line_spacing;
        let desired_size = vec2(
            galley.size.x.max(desired_width.min(available_width)),
            galley.size.y.max(desired_height),
        );
        let rect = ui.allocate_space(desired_size);
        let sense = if enabled {
            Sense::click_and_drag()
        } else {
            Sense::nothing()
        };
        let response = ui.interact(rect, id, sense); // TODO: implement drag-select

        if response.clicked && enabled {
            ui.memory().request_kb_focus(id);
            if let Some(mouse_pos) = ui.input().mouse.pos {
                state.cursor = Some(galley.char_at(mouse_pos - response.rect.min).char_idx);
            }
        } else if ui.input().mouse.click || (ui.input().mouse.pressed && !response.hovered) {
            // User clicked somewhere else
            ui.memory().surrender_kb_focus(id);
        }
        if !enabled {
            ui.memory().surrender_kb_focus(id);
        }

        if response.hovered && enabled {
            ui.output().cursor_icon = CursorIcon::Text;
        }

        if ui.memory().has_kb_focus(id) && enabled {
            let mut cursor = state.cursor.unwrap_or_else(|| text.chars().count());
            cursor = clamp(cursor, 0..=text.chars().count());

            for event in &ui.input().events {
                match event {
                    Event::Copy | Event::Cut => {
                        // TODO: cut
                        ui.ctx().output().copied_text = text.clone();
                    }
                    Event::Text(text_to_insert) => {
                        // newlines are handled by `Key::Enter`.
                        if text_to_insert != "\n" && text_to_insert != "\r" {
                            insert_text(&mut cursor, text, text_to_insert);
                        }
                    }
                    Event::Key {
                        key: Key::Enter,
                        pressed: true,
                    } => {
                        if multiline {
                            insert_text(&mut cursor, text, "\n");
                        } else {
                            // Common to end input with enter
                            ui.memory().surrender_kb_focus(id);
                            break;
                        }
                    }
                    Event::Key {
                        key: Key::Escape,
                        pressed: true,
                    } => {
                        ui.memory().surrender_kb_focus(id);
                        break;
                    }
                    Event::Key { key, pressed: true } => {
                        on_key_press(&mut cursor, text, *key);
                    }
                    _ => {}
                }
            }
            state.cursor = Some(cursor);

            // layout again to avoid frame delay:
            let font = &ui.fonts()[text_style];
            galley = if multiline {
                font.layout_multiline(text.clone(), available_width)
            } else {
                font.layout_single_line(text.clone())
            };

            // dbg!(&galley);
        }

        let painter = ui.painter();
        let visuals = ui.style().interact(&response);

        {
            let bg_rect = response.rect.expand(2.0); // breathing room for content
            painter.add(PaintCmd::Rect {
                rect: bg_rect,
                corner_radius: visuals.corner_radius,
                fill: ui.style().visuals.dark_bg_color,
                // fill: visuals.bg_fill,
                stroke: visuals.bg_stroke,
            });
        }

        if ui.memory().has_kb_focus(id) {
            let cursor_blink_hz = ui.style().visuals.cursor_blink_hz;
            let show_cursor = if 0.0 < cursor_blink_hz {
                ui.ctx().request_repaint(); // TODO: only when cursor blinks on or off
                (ui.input().time * cursor_blink_hz as f64 * 3.0).floor() as i64 % 3 != 0
            } else {
                true
            };

            if show_cursor {
                if let Some(cursor) = state.cursor {
                    let cursor_pos = response.rect.min + galley.char_start_pos(cursor);
                    painter.line_segment(
                        [cursor_pos, cursor_pos + vec2(0.0, line_spacing)],
                        (ui.style().visuals.text_cursor_width, color::WHITE),
                    );
                }
            }
        }

        let text_color = text_color
            .or(ui.style().visuals.override_text_color)
            .unwrap_or_else(|| visuals.text_color());
        painter.galley(response.rect.min, galley, text_style, text_color);
        ui.memory().text_edit.insert(id, state);

        Response {
            lost_kb_focus: ui.memory().lost_kb_focus(id), // we may have lost it during the course of this function
            ..response
        }
    }
}

fn insert_text(cursor: &mut usize, text: &mut String, text_to_insert: &str) {
    // eprintln!("insert_text {:?}", text_to_insert);

    let mut char_it = text.chars();
    let mut new_text = String::with_capacity(text.capacity());
    for _ in 0..*cursor {
        let c = char_it.next().unwrap();
        new_text.push(c);
    }
    *cursor += text_to_insert.chars().count();
    new_text += text_to_insert;
    new_text.extend(char_it);
    *text = new_text;
}

fn on_key_press(cursor: &mut usize, text: &mut String, key: Key) {
    // eprintln!("on_key_press before: '{}', cursor at {}", text, cursor);

    match key {
        Key::Backspace if *cursor > 0 => {
            *cursor -= 1;

            let mut char_it = text.chars();
            let mut new_text = String::with_capacity(text.capacity());
            for _ in 0..*cursor {
                new_text.push(char_it.next().unwrap())
            }
            new_text.extend(char_it.skip(1));
            *text = new_text;
        }
        Key::Delete => {
            let mut char_it = text.chars();
            let mut new_text = String::with_capacity(text.capacity());
            for _ in 0..*cursor {
                new_text.push(char_it.next().unwrap())
            }
            new_text.extend(char_it.skip(1));
            *text = new_text;
        }
        Key::Enter => {} // handled earlier
        Key::Home => {
            // To start of paragraph:
            let pos = line_col_from_char_idx(text, *cursor);
            *cursor = char_idx_from_line_col(text, (pos.0, 0));
        }
        Key::End => {
            // To end of paragraph:
            let pos = line_col_from_char_idx(text, *cursor);
            let line = line_from_number(text, pos.0);
            *cursor = char_idx_from_line_col(text, (pos.0, line.chars().count()));
        }
        Key::Left if *cursor > 0 => {
            *cursor -= 1;
        }
        Key::Right => {
            *cursor = (*cursor + 1).min(text.chars().count());
        }
        Key::Up => {
            let mut pos = line_col_from_char_idx(text, *cursor);
            pos.0 = pos.0.saturating_sub(1);
            *cursor = char_idx_from_line_col(text, pos);
        }
        Key::Down => {
            let mut pos = line_col_from_char_idx(text, *cursor);
            pos.0 += 1;
            *cursor = char_idx_from_line_col(text, pos);
        }
        _ => {}
    }

    // eprintln!("on_key_press after:  '{}', cursor at {}\n", text, cursor);
}

fn line_col_from_char_idx(s: &str, char_idx: usize) -> (usize, usize) {
    let mut char_count = 0;

    let mut last_line_nr = 0;
    let mut last_line = s;
    for (line_nr, line) in s.split('\n').enumerate() {
        let line_width = line.chars().count();
        if char_idx <= char_count + line_width {
            return (line_nr, char_idx - char_count);
        }
        char_count += line_width + 1;
        last_line_nr = line_nr;
        last_line = line;
    }

    // safe fallback:
    (last_line_nr, last_line.chars().count())
}

fn char_idx_from_line_col(s: &str, pos: (usize, usize)) -> usize {
    let mut char_count = 0;
    for (line_nr, line) in s.split('\n').enumerate() {
        if line_nr == pos.0 {
            return char_count + pos.1.min(line.chars().count());
        }
        char_count += line.chars().count() + 1;
    }
    char_count
}

fn line_from_number(s: &str, desired_line_number: usize) -> &str {
    for (line_nr, line) in s.split('\n').enumerate() {
        if line_nr == desired_line_number {
            return line;
        }
    }
    s
}
