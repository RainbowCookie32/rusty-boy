use imgui::*;

const FADE_DURATION: f64 = 0.5;
const NOTIFICATION_OPACITY: f64 = 0.7;
const NOTIFICATION_SHOW_DURATION: f64 = 3.5;
const NOTIFICATION_TOTAL_DURATION: f64 = FADE_DURATION + NOTIFICATION_SHOW_DURATION + FADE_DURATION;

pub struct Notification {
    title: ImString,
    content: ImString,
    
    created_at: f64
}

impl Notification {
    pub fn init(title: ImString, content: ImString, created_at: f64) -> Notification {
        Notification {
            title,
            content,

            created_at
        }
    }

    pub fn draw(&mut self, ui: &Ui, idx: usize) -> bool {
        let elapsed = ui.time() - self.created_at;
        let mut flags = WindowFlags::empty();

        flags.set(WindowFlags::NO_MOVE, true);
        flags.set(WindowFlags::NO_DECORATION, true);

        let opacity = {
            if elapsed <= FADE_DURATION {
                (elapsed / FADE_DURATION) * NOTIFICATION_OPACITY
            }
            else if elapsed > NOTIFICATION_SHOW_DURATION + FADE_DURATION {
                (1.0 + ((NOTIFICATION_TOTAL_DURATION - FADE_DURATION - elapsed) / FADE_DURATION)) * NOTIFICATION_OPACITY
            }
            else {
                1.0 * NOTIFICATION_OPACITY
            }
        };

        let title = ImString::from(format!("Notification {}", idx));

        let window = Window::new(&title)
            .flags(flags)    
            .bg_alpha(opacity as f32)
            .size([400.0, 55.0], Condition::Always)
            .position([1280.0, 768.0 - (60.0 * idx as f32)], Condition::Always)
            .position_pivot([1.0, 1.0])
        ;

        window.build(ui, || {
            ui.text(&self.title);
            ui.separator();
            ui.text_wrapped(&self.content);
        });

        elapsed > NOTIFICATION_TOTAL_DURATION
    }
}
