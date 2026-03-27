use rex_common::color;
use starbase_console::{ConsoleStream, ConsoleStreamType, Reporter};
use starbase_styles::color::owo::{OwoColorize, XtermColors};
use starbase_styles::color::{Color, OwoStyle, no_color};

#[derive(Clone, Debug, PartialEq)]
pub enum Level {
    Zero,
    One,
    Two,
    Three,
}

impl Level {
    pub fn is(&self, level: Level) -> bool {
        match level {
            Self::Zero => false,
            Self::One => matches!(self, Level::One | Level::Two | Level::Three),
            Self::Two => matches!(self, Level::Two | Level::Three),
            Self::Three => matches!(self, Level::Three),
        }
    }
}

const STEP_CHAR: &str = "▮"; // ■ ▪
const CACHED_COLORS: [u8; 4] = [57, 63, 69, 75]; // blue
const PASSED_COLORS: [u8; 4] = [35, 42, 49, 86]; // green
const FAILED_COLORS: [u8; 4] = [124, 125, 126, 127]; // red
const MUTED_COLORS: [u8; 4] = [240, 242, 244, 246]; // gray
const SETUP_COLORS: [u8; 4] = [198, 205, 212, 219]; // pink
const ANNOUNCEMENT_COLORS: [u8; 4] = [208, 214, 220, 226]; // yellow

#[derive(Clone, Copy)]
pub enum Checkpoint {
    Announcement,
    Cached,
    Default,
    Failed,
    Passed,
    Setup,
}

fn bold(message: &str) -> String {
    if no_color() {
        message.to_owned()
    } else {
        OwoStyle::new().style(message).bold().to_string()
    }
}

#[derive(Debug)]
pub struct RexReporter {
    err: ConsoleStream,
    out: ConsoleStream,
    test_mode: bool,
}

impl RexReporter {
    pub fn new_testing() -> Self {
        Self {
            err: ConsoleStream::new_testing(ConsoleStreamType::Stderr),
            out: ConsoleStream::new_testing(ConsoleStreamType::Stdout),
            test_mode: true,
        }
    }
}

impl Default for RexReporter {
    fn default() -> Self {
        Self {
            err: ConsoleStream::empty(ConsoleStreamType::Stderr),
            out: ConsoleStream::empty(ConsoleStreamType::Stdout),
            test_mode: false,
        }
    }
}

impl Reporter for RexReporter {
    fn inherit_streams(&mut self, err: ConsoleStream, out: ConsoleStream) {
        if !self.test_mode {
            self.err = err;
            self.out = out;
        }
    }
}

impl RexReporter {
    fn format_block(&self, label: &str, bg: u8) -> String {
        let body = format!(" {} ", label.to_uppercase());

        if no_color() {
            body
        } else {
            OwoStyle::new()
                .style(body)
                .bold()
                .color(XtermColors::from(Color::Black as u8))
                .on_color(XtermColors::from(bg))
                .to_string()
        }
    }

    pub fn format_checkpoint<M: AsRef<str>, C: AsRef<[String]>>(
        &self,
        checkpoint: Checkpoint,
        message: M,
        comments: C,
    ) -> String {
        let colors = match checkpoint {
            Checkpoint::Announcement => ANNOUNCEMENT_COLORS,
            Checkpoint::Cached => CACHED_COLORS,
            Checkpoint::Failed => FAILED_COLORS,
            Checkpoint::Passed => PASSED_COLORS,
            Checkpoint::Default => MUTED_COLORS,
            Checkpoint::Setup => SETUP_COLORS,
        };

        let mut out = format!(
            "{}{}{}{} {}",
            color::paint(colors[0], STEP_CHAR),
            color::paint(colors[1], STEP_CHAR),
            color::paint(colors[2], STEP_CHAR),
            color::paint(colors[3], STEP_CHAR),
            bold(message.as_ref()),
        );

        let suffix = self.format_comments(comments);

        if !suffix.is_empty() {
            out.push(' ');
            out.push_str(&suffix);
        }

        out
    }

    pub fn format_comments<C: AsRef<[String]>>(&self, comments: C) -> String {
        let comments = comments.as_ref();

        if comments.is_empty() {
            return String::new();
        }

        color::muted(format!("({})", comments.join(", ")))
    }

    pub fn format_entry_key<K: AsRef<str>>(&self, key: K) -> String {
        color::muted_light(format!("{}:", key.as_ref()))
    }

    pub fn print_checkpoint<M: AsRef<str>>(
        &self,
        checkpoint: Checkpoint,
        message: M,
    ) -> miette::Result<()> {
        self.print_checkpoint_with_comments(checkpoint, message, &[])
    }

    pub fn print_checkpoint_with_comments<M: AsRef<str>, C: AsRef<[String]>>(
        &self,
        checkpoint: Checkpoint,
        message: M,
        comments: C,
    ) -> miette::Result<()> {
        if !self.out.is_quiet() {
            self.out
                .write_line(self.format_checkpoint(checkpoint, message, comments))?;
        }

        Ok(())
    }

    pub fn print_entry<K: AsRef<str>, V: AsRef<str>>(
        &self,
        key: K,
        value: V,
    ) -> miette::Result<()> {
        self.out
            .write_line(format!("{} {}", self.format_entry_key(key), value.as_ref()))?;

        Ok(())
    }

    pub fn print_header<M: AsRef<str>>(&self, message: M) -> miette::Result<()> {
        self.out.write_newline()?;
        self.out
            .write_line(self.format_block(message.as_ref(), Color::Purple as u8))?;
        self.out.write_newline()?;

        Ok(())
    }
}
