use rex_env_var::GlobalEnvBag;
use starbase_styles::color::{no_color, supports_color};

pub fn setup_no_colors() {
    let bag = GlobalEnvBag::instance();
    bag.set("NO_COLOR", "1");
    // https://github.com/mitsuhiko/clicolors-control/issues/19
    bag.set("CLICOLOR", "0");
    bag.remove("FORCE_COLOR");
}

pub fn setup_colors(force: bool) {
    let bag = GlobalEnvBag::instance();

    // If being forced by --color or other env vars
    if force || bag.has("REX_COLOR") || bag.has("FORCE_COLOR") || bag.has("CLICOLOR_FORCE") {
        let mut color_level = bag
            .get("REX_COLOR")
            .or_else(|| bag.get("FORCE_COLOR"))
            .unwrap_or("3".to_owned());

        // https://nodejs.org/api/cli.html#force_color1-2-3
        if color_level.is_empty() || color_level == "true" {
            color_level = "1".to_owned();
        } else if color_level == "false" {
            color_level = "0".to_owned();
        }

        if color_level == "0" {
            setup_no_colors();
        } else {
            // https://bixense.com/clicolors/
            bag.set("CLICOLOR_FORCE", &color_level);
            bag.set("FORCE_COLOR", &color_level);
            bag.remove("NO_COLOR");
        }

        return;
    }

    if no_color() {
        setup_no_colors();
    } else {
        bag.set("CLICOLOR", supports_color().to_string());
    }
}
