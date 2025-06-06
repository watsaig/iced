use iced::widget::{
    button, center, checkbox, column, container, horizontal_rule, pick_list,
    progress_bar, row, scrollable, slider, text, text_input, toggler,
    vertical_rule, vertical_space,
};
use iced::window::{self, settings::PlatformSpecific};
use iced::{Center, Color, Element, Fill, Subscription, Theme};
use iced::{Task, keyboard};

const DEFAULT_TRANSPARENCY: bool = true;
const DEFAULT_BLUR: bool = true;

pub fn main() -> iced::Result {
    let win_settings = window::Settings {
        platform_specific: PlatformSpecific {
            #[cfg(target_os = "macos")]
            blur_radius: 60,
            ..Default::default()
        },
        blur: DEFAULT_BLUR,
        transparent: DEFAULT_TRANSPARENCY,
        ..Default::default()
    };
    iced::application(Styling::default, Styling::update, Styling::view)
        .subscription(Styling::subscription)
        .window(win_settings)
        .theme(Styling::theme)
        .run()
}

struct Styling {
    theme: Theme,
    input_value: String,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
}

impl Default for Styling {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            input_value: String::new(),
            slider_value: 0.,
            checkbox_value: DEFAULT_BLUR,
            toggler_value: DEFAULT_TRANSPARENCY,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    InputChanged(String),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    PreviousTheme,
    NextTheme,
}

impl Styling {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = theme;
            }
            Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => {
                self.checkbox_value = value;
                return window::get_latest()
                    .and_then(move |id| window::set_blur(id, value));
            }
            Message::TogglerToggled(value) => {
                self.toggler_value = value;
                return window::get_latest()
                    .and_then(move |id| window::set_transparent(id, value));
            }
            Message::PreviousTheme | Message::NextTheme => {
                if let Some(current) = Theme::ALL
                    .iter()
                    .position(|candidate| &self.theme == candidate)
                {
                    self.theme = if matches!(message, Message::NextTheme) {
                        Theme::ALL[(current + 1) % Theme::ALL.len()].clone()
                    } else if current == 0 {
                        Theme::ALL
                            .last()
                            .expect("Theme::ALL must not be empty")
                            .clone()
                    } else {
                        Theme::ALL[current - 1].clone()
                    };
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = column![
            text("Theme:"),
            pick_list(Theme::ALL, Some(&self.theme), Message::ThemeChanged)
                .width(Fill),
        ]
        .spacing(10);

        let text_input = text_input("Type something...", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(20);

        let styled_button = |label| {
            button(text(label).width(Fill).center())
                .padding(10)
                .on_press(Message::ButtonPressed)
        };

        let primary = styled_button("Primary");
        let success = styled_button("Success").style(button::success);
        let warning = styled_button("Warning").style(button::warning);
        let danger = styled_button("Danger").style(button::danger);

        let slider =
            || slider(0.0..=100.0, self.slider_value, Message::SliderChanged);

        let progress_bar = || progress_bar(0.0..=100.0, self.slider_value);

        let scrollable = scrollable(column![
            "Scroll me!",
            vertical_space().height(800),
            "You did it!"
        ])
        .width(Fill)
        .height(100);

        let checkbox = checkbox("Toggle blur", self.checkbox_value)
            .on_toggle(Message::CheckboxToggled);

        let toggler = toggler(self.toggler_value)
            .label("Toggle transparency")
            .on_toggle(Message::TogglerToggled)
            .spacing(10);

        let card = {
            container(
                column![
                    text("Card Example").size(24),
                    slider(),
                    progress_bar(),
                ]
                .spacing(20),
            )
            .width(Fill)
            .padding(20)
            .style(container::bordered_box)
        };

        let content = column![
            choose_theme,
            horizontal_rule(38),
            text_input,
            row![primary, success, warning, danger]
                .spacing(10)
                .align_y(Center),
            slider(),
            progress_bar(),
            row![
                scrollable,
                vertical_rule(38),
                column![checkbox, toggler].spacing(20)
            ]
            .spacing(10)
            .height(100)
            .align_y(Center),
            card
        ]
        .spacing(20)
        .padding(20)
        .max_width(600);

        // Translucent container for the main content, surrounded by a fully transparent container
        let content =
            container(content).center_x(Fill).style(|_theme: &Theme| {
                container::Style {
                    background: Some(
                        Color::from_rgba(0.9, 0.9, 0.9, 0.5).into(),
                    ),
                    ..Default::default()
                }
            });

        center(content)
            .padding(50)
            .style(container::transparent)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, _modifiers| match key {
            keyboard::Key::Named(
                keyboard::key::Named::ArrowUp | keyboard::key::Named::ArrowLeft,
            ) => Some(Message::PreviousTheme),
            keyboard::Key::Named(
                keyboard::key::Named::ArrowDown
                | keyboard::key::Named::ArrowRight,
            ) => Some(Message::NextTheme),
            _ => None,
        })
    }

    fn theme(&self) -> Theme {
        let bg = self
            .theme
            .palette()
            .background
            .scale_alpha(self.slider_value / 100.0);
        let palette = iced::theme::Palette {
            background: bg,
            ..self.theme.palette()
        };
        iced::Theme::custom("custom".to_string(), palette)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;

    use iced_test::{Error, simulator};

    #[test]
    #[ignore]
    fn it_showcases_every_theme() -> Result<(), Error> {
        Theme::ALL
            .par_iter()
            .cloned()
            .map(|theme| {
                let mut styling = Styling::default();
                let _ = styling.update(Message::ThemeChanged(theme));

                let theme = styling.theme();

                let mut ui = simulator(styling.view());
                let snapshot = ui.snapshot(&theme)?;

                assert!(
                    snapshot.matches_hash(format!(
                        "snapshots/{theme}",
                        theme = theme
                            .to_string()
                            .to_ascii_lowercase()
                            .replace(" ", "_")
                    ))?,
                    "snapshots for {theme} should match!"
                );

                Ok(())
            })
            .collect()
    }
}
