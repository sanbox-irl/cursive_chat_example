//! Handle colors and themes in the UI.
//!
//! # Color palette
//!
//! To achieve a customizable, yet unified look, Cursive uses a configurable
//! palette of colors to be used through the entire application.
//!
//! These colors are:
//!
//! * **`background`**: used to color the application background
//!   (around views).
//!   Defaults to **blue**.
//! * **`shadow`**: used to color shadow around views.
//!   Defaults to **black**.
//! * **`view`**: used to color the background for views.
//!   Defaults to **white**.
//! * **`primary`**: used to print primary text.
//!   Defaults to **black**.
//! * **`secondary`**: used to print secondary text.
//!   Defaults to **blue**.
//! * **`tertiary`**: used to print tertiary text.
//!   Defaults to **white**.
//! * **`title_primary`**: used to print primary titles.
//!   Defaults to **red**.
//! * **`title_secondary`**: used to print secondary titles.
//!   Defaults to **yellow**.
//! * **`highlight`**: used to highlight selected items.
//!   Defaults to **red**.
//! * **`highlight_inactive`**: used to highlight selected but inactive items.
//!   Defaults to **blue**.
//!
//! # Color Styles
//!
//! Each cell of the terminal uses two colors: *foreground* and *background*.
//!
//! Color styles are defined to easily refer to a pair of colors from the
//! palette.
//!
//! * **`Background`**: style used to print the application background.
//!     * Its *background* color is `background`.
//!     * Its *foreground* color is unimportant as no characters are ever
//!       printed in the background.
//! * **`Shadow`**: style used to print shadows behind views.
//!     * Its *background* color is `shadow`.
//!     * Here again, the *foreground* color is unimportant.
//! * **`Primary`**: style used to print primary text.
//!     * Its *background* color is `view`.
//!     * Its *foreground* color is `primary`.
//! * **`Secondary`**: style used to print secondary text.
//!     * Its *background* color is `view`.
//!     * Its *foreground* color is `secondary`.
//! * **`Tertiary`**: style used to print tertiary text.
//!     * Its *background* color is `view`.
//!     * Its *foreground* color is `tertiary`.
//! * **`TitlePrimary`**: style used to print titles.
//!     * Its *background* color is `view`.
//!     * Its *foreground* color is `title_primary`.
//! * **`TitleSecondary`**: style used to print secondary titles.
//!     * Its *background* color is `view`.
//!     * Its *foreground* color is `title_secondary`.
//! * **`Highlight`**: style used to print selected items.
//!     * Its *background* color is `highlight`.
//!     * Its *foreground* color is `view`.
//! * **`HighlightInactive`**: style used to print selected,
//!   but inactive items.
//!     * Its *background* color is `highlight_inactive`.
//!     * Its *foreground* color is `view`.
//!
//! Using one of these pairs when styling your application helps give it a
//! coherent look.
//!
//! # Effects
//!
//! On top of a color style, some effects can be applied on cells: `Reverse`,
//! for instance, swaps the foreground and background colors of a cell.
//!
//! # Themes
//!
//! A theme defines the color palette an application will use, as well as
//! various options to style views.
//!
//! Themes are described in toml configuration files. All fields are optional.
//!
//! Here are the possible entries:
//!
//! ```toml
//! # Every field in a theme file is optional.
//!
//! # First come some various options
//! shadow = false  # Don't draw shadows around stacked views
//! borders = "simple"  # Alternatives are "none" and "outset"
//!
//! # Here we define the color palette.
//! [colors]
//! 	background = "black"
//! 	# If the value is an array, the first valid color will be used.
//! 	# If the terminal doesn't support custom color,
//! 	# non-base colors will be skipped.
//! 	shadow     = ["#000000", "black"]
//! 	view       = "#d3d7cf"
//!
//! 	# Array and simple values have the same effect.
//! 	primary   = ["#111111"]
//! 	secondary = "#EEEEEE"
//! 	tertiary  = "#444444"
//!
//! 	# Hex values can use lower or uppercase.
//! 	# (base color MUST be lowercase)
//! 	title_primary   = "#ff5555"
//! 	title_secondary = "#ffff55"
//!
//! 	# Lower precision values can use only 3 digits.
//! 	highlight          = "#F00"
//! 	highlight_inactive = "#5555FF"
//! ```

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use toml;

/// Text effect
#[derive(Clone, Copy, Debug)]
pub enum Effect {
    /// No effect
    Simple,
    /// Reverses foreground and background colors
    Reverse,
    // TODO: bold, italic, underline
}

/// Combines a front and back color.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub struct ColorPair {
    /// Color used for the foreground.
    pub front: Color,
    /// Color used for the background.
    pub back: Color,
}

impl ColorPair {
    /// Return an inverted color pair.
    ///
    /// With swapped front abd back color.
    pub fn invert(&self) -> Self {
        ColorPair {
            front: self.back,
            back: self.front,
        }
    }

    /// Creates a new color pair from color IDs.
    pub fn from_256colors(front: u8, back: u8) -> Self {
        Self {
            front: Color::from_256colors(front),
            back: Color::from_256colors(back),
        }
    }
}

/// Possible color style for a cell.
///
/// Represents a color pair role to use when printing something.
///
/// The current theme will assign each role a foreground and background color.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum ColorStyle {
    /// Style set by terminal before entering a Cursive program.
    Default,
    /// Application background, where no view is present.
    Background,
    /// Color used by view shadows. Only background matters.
    Shadow,
    /// Main text with default background.
    Primary,
    /// Secondary text color, with default background.
    Secondary,
    /// Tertiary text color, with default background.
    Tertiary,
    /// Title text color with default background.
    TitlePrimary,
    /// Alternative color for a title.
    TitleSecondary,
    /// Alternate text with highlight background.
    Highlight,
    /// Highlight color for inactive views (not in focus).
    HighlightInactive,
    /// Directly specifies colors, independently of the theme.
    Custom {
        /// Foreground color
        front: Color,
        /// Background color
        back: Color,
    },
}

impl ColorStyle {
    /// Return the color pair that this style represents.
    ///
    /// Returns `(front, back)`.
    pub fn resolve(&self, theme: &Theme) -> ColorPair {
        let c = &theme.colors;
        let (front, back) = match *self {
            ColorStyle::Default => (Color::Default, Color::Default),
            ColorStyle::Background => (c.view, c.background),
            ColorStyle::Shadow => (c.shadow, c.shadow),
            ColorStyle::Primary => (c.primary, c.view),
            ColorStyle::Secondary => (c.secondary, c.view),
            ColorStyle::Tertiary => (c.tertiary, c.view),
            ColorStyle::TitlePrimary => (c.title_primary, c.view),
            ColorStyle::TitleSecondary => (c.title_secondary, c.view),
            ColorStyle::Highlight => (c.view, c.highlight),
            ColorStyle::HighlightInactive => (c.view, c.highlight_inactive),
            ColorStyle::Custom { front, back } => (front, back),
        };
        ColorPair { front, back }
    }
}

/// Represents the style a Cursive application will use.
#[derive(Clone,Debug)]
pub struct Theme {
    /// Whether views in a StackView should have shadows.
    pub shadow: bool,
    /// How view borders should be drawn.
    pub borders: BorderStyle,
    /// What colors should be used through the application?
    pub colors: Palette,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            shadow: true,
            borders: BorderStyle::Simple,
            colors: Palette {
                background: Color::Dark(BaseColor::Blue),
                shadow: Color::Dark(BaseColor::Black),
                view: Color::Dark(BaseColor::White),
                primary: Color::Dark(BaseColor::Black),
                secondary: Color::Dark(BaseColor::Blue),
                tertiary: Color::Light(BaseColor::White),
                title_primary: Color::Dark(BaseColor::Red),
                title_secondary: Color::Dark(BaseColor::Yellow),
                highlight: Color::Dark(BaseColor::Red),
                highlight_inactive: Color::Dark(BaseColor::Blue),
            },
        }
    }
}

impl Theme {
    fn load(&mut self, table: &toml::value::Table) {
        if let Some(&toml::Value::Boolean(shadow)) = table.get("shadow") {
            self.shadow = shadow;
        }

        if let Some(&toml::Value::String(ref borders)) = table.get("borders") {
            self.borders = BorderStyle::from(borders);
        }

        if let Some(&toml::Value::Table(ref table)) = table.get("colors") {
            self.colors.load(table);
        }
    }
}

/// Specifies how some borders should be drawn.
///
/// Borders are used around Dialogs, select popups, and panels.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum BorderStyle {
    /// Simple borders.
    Simple,
    /// Outset borders with a simple 3d effect.
    Outset,
    /// No borders.
    None,
}

impl BorderStyle {
    fn from(s: &str) -> Self {
        if s == "simple" {
            BorderStyle::Simple
        } else if s == "outset" {
            BorderStyle::Outset
        } else {
            BorderStyle::None
        }
    }
}

/// Color configuration for the application.
///
/// Assign each color role an actual color.
#[derive(Copy,Clone,Debug)]
pub struct Palette {
    /// Color used for the application background.
    pub background: Color,
    /// Color used for View shadows.
    pub shadow: Color,
    /// Color used for View backgrounds.
    pub view: Color,
    /// Primary color used for the text.
    pub primary: Color,
    /// Secondary color used for the text.
    pub secondary: Color,
    /// Tertiary color used for the text.
    pub tertiary: Color,
    /// Primary color used for title text.
    pub title_primary: Color,
    /// Secondary color used for title text.
    pub title_secondary: Color,
    /// Color used for highlighting text.
    pub highlight: Color,
    /// Color used for highlighting inactive text.
    pub highlight_inactive: Color,
}

impl Palette {
    /// Fills `self` with the colors from the given `table`.
    fn load(&mut self, table: &toml::value::Table) {
        load_color(&mut self.background, table.get("background"));
        load_color(&mut self.shadow, table.get("shadow"));
        load_color(&mut self.view, table.get("view"));
        load_color(&mut self.primary, table.get("primary"));
        load_color(&mut self.secondary, table.get("secondary"));
        load_color(&mut self.tertiary, table.get("tertiary"));
        load_color(&mut self.title_primary, table.get("title_primary"));
        load_color(&mut self.title_secondary, table.get("title_secondary"));
        load_color(&mut self.highlight, table.get("highlight"));
        load_color(&mut self.highlight_inactive,
                   table.get("highlight_inactive"));
    }
}

/// Parses `value` and fills `target` if it's a valid color.
fn load_color(target: &mut Color, value: Option<&toml::Value>) -> bool {
    if let Some(value) = value {
        match *value {
            toml::Value::String(ref value) => {
                if let Some(color) = Color::parse(value) {
                    *target = color;
                    true
                } else {
                    false
                }
            }
            toml::Value::Array(ref array) => {
                array.iter().any(|item| load_color(target, Some(item)))
            }
            _ => false,
        }
    } else {
        false
    }
}

/// One of the 8 base colors.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum BaseColor {
    /// Black color
    ///
    /// Color #0
    Black,
    /// Red color
    ///
    /// Color #1
    Red,
    /// Green color
    ///
    /// Color #2
    Green,
    /// Yellow color (Red + Green)
    ///
    /// Color #3
    Yellow,
    /// Blue color
    ///
    /// Color #4
    Blue,
    /// Magenta color (Red + Blue)
    ///
    /// Color #5
    Magenta,
    /// Cyan color (Green + Blue)
    ///
    /// Color #6
    Cyan,
    /// White color (Red + Green + Blue)
    ///
    /// Color #7
    White,
}

impl From<u8> for BaseColor {
    fn from(n: u8) -> Self {
        match n % 8 {
            0 => BaseColor::Black,
            1 => BaseColor::Red,
            2 => BaseColor::Green,
            3 => BaseColor::Yellow,
            4 => BaseColor::Blue,
            5 => BaseColor::Magenta,
            6 => BaseColor::Cyan,
            7 => BaseColor::White,
            _ => unreachable!(),
        }
    }
}

/// Represents a color used by the theme.
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum Color {
    /// Represents a color, preset by terminal.
    Default,
    /// One of the 8 base colors.
    Dark(BaseColor),
    /// Lighter version of a base color.
    Light(BaseColor),
    /// True-color, 24-bit.
    Rgb(u8, u8, u8),
    /// Low-resolution
    ///
    /// Each value should be `<= 5` (you'll get panics otherwise).
    ///
    /// These 216 possible colors are part of the default color palette.
    RgbLowRes(u8, u8, u8),
}

/// Possible error returned when loading a theme.
#[derive(Debug)]
pub enum Error {
    /// An error occured when reading the file.
    Io(io::Error),
    /// An error occured when parsing the toml content.
    Parse(toml::de::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Parse(err)
    }
}

impl Color {
    /// Creates a color from its ID in the 256 colors list.
    ///
    /// * Colors 0-7 are base dark colors.
    /// * Colors 8-15 are base light colors.
    /// * Colors 16-255 are rgb colors with 6 values per channel.
    pub fn from_256colors(n: u8) -> Self {
        if n < 8 {
            Color::Dark(BaseColor::from(n))
        } else if n < 16 {
            Color::Light(BaseColor::from(n))
        } else {
            let n = n - 16;

            let r = n / 36;
            let g = (n % 36) / 6;
            let b = n % 6;

            Color::RgbLowRes(r, g, b)
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
                 "black" => Color::Dark(BaseColor::Black),
                 "red" => Color::Dark(BaseColor::Red),
                 "green" => Color::Dark(BaseColor::Green),
                 "yellow" => Color::Dark(BaseColor::Yellow),
                 "blue" => Color::Dark(BaseColor::Blue),
                 "magenta" => Color::Dark(BaseColor::Magenta),
                 "cyan" => Color::Dark(BaseColor::Cyan),
                 "white" => Color::Dark(BaseColor::White),
                 "light black" => Color::Light(BaseColor::Black),
                 "light red" => Color::Light(BaseColor::Red),
                 "light green" => Color::Light(BaseColor::Green),
                 "light yellow" => Color::Light(BaseColor::Yellow),
                 "light blue" => Color::Light(BaseColor::Blue),
                 "light magenta" => Color::Light(BaseColor::Magenta),
                 "light cyan" => Color::Light(BaseColor::Cyan),
                 "light white" => Color::Light(BaseColor::White),
                 value => return Color::parse_special(value),
             })
    }

    fn parse_special(value: &str) -> Option<Color> {
        if value.starts_with('#') {

            let value = &value[1..];
            // Compute per-color length, and amplitude
            let (l, multiplier) = match value.len() {
                6 => (2, 1),
                3 => (1, 17),
                _ => panic!("Cannot parse color: {}", value),
            };
            let r = load_hex(&value[0..l]) * multiplier;
            let g = load_hex(&value[l..2 * l]) * multiplier;
            let b = load_hex(&value[2 * l..3 * l]) * multiplier;
            Some(Color::Rgb(r as u8, g as u8, b as u8))
        } else if value.len() == 3 {
            // RGB values between 0 and 5 maybe?
            let rgb: Vec<_> =
                value.chars().map(|c| c as i16 - '0' as i16).collect();
            if rgb.iter().all(|&i| i >= 0 && i < 6) {
                Some(Color::RgbLowRes(rgb[0] as u8,
                                      rgb[1] as u8,
                                      rgb[2] as u8))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Loads a theme from file and sets it as active.
pub fn load_theme_file<P: AsRef<Path>>(filename: P) -> Result<Theme, Error> {
    let content = {
        let mut content = String::new();
        let mut file = try!(File::open(filename));
        try!(file.read_to_string(&mut content));
        content
    };

    load_theme(&content)
}

/// Loads a theme string and sets it as active.
pub fn load_theme(content: &str) -> Result<Theme, Error> {
    let table = toml::de::from_str(content)?;

    let mut theme = Theme::default();
    theme.load(&table);

    Ok(theme)
}

/// Loads the default theme, and returns its representation.
pub fn load_default() -> Theme {
    Theme::default()
}

/// Loads a hexadecimal code
fn load_hex(s: &str) -> u16 {
    let mut sum = 0;
    for c in s.chars() {
        sum *= 16;
        sum += match c {
            n @ '0'...'9' => n as i16 - '0' as i16,
            n @ 'a'...'f' => n as i16 - 'a' as i16 + 10,
            n @ 'A'...'F' => n as i16 - 'A' as i16 + 10,
            _ => 0,
        };
    }

    sum as u16
}
