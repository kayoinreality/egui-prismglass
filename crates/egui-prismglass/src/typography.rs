//! Typography for `egui-prismglass`: the bundled [Inter] typeface in three weights plus
//! tabular (fixed-width) figures for numbers that must line up.
//!
//! `egui` has no notion of font weight, so this module registers the bundled Inter variable
//! font several times — once per weight (via the `wght` variation axis) — under named
//! [`FontFamily`]s. Widgets ask for a weight with [`prism_font`] and transparently fall back to
//! the regular proportional font when the bundled fonts are not installed (or when the `inter`
//! cargo feature is disabled), so nothing ever panics on a missing family.
//!
//! ```no_run
//! # fn setup(ctx: &egui::Context) {
//! // Once during setup (e.g. in your `eframe` creation closure):
//! egui_prismglass::install_fonts(ctx);
//! # }
//! ```
//!
//! [Inter]: https://rsms.me/inter/

use egui::{Context, FontFamily, FontId, Id};

#[cfg(feature = "inter")]
use egui::epaint::text::VariationCoords;
#[cfg(feature = "inter")]
use egui::{FontData, FontInsert, FontPriority, FontTweak, InsertFontFamily};

const FONTS_INSTALLED_ID: &str = "egui_prismglass_fonts_installed";

#[cfg(feature = "inter")]
const INTER_VARIABLE: &[u8] = include_bytes!("../fonts/InterVariable.ttf");
#[cfg(feature = "inter")]
const INTER_TABULAR_DIGITS: &[u8] = include_bytes!("../fonts/InterTabularDigits.ttf");

/// Font weight used by `PrismGlass` widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FontWeight {
    /// Body copy (`400`).
    #[default]
    Regular,
    /// Labels, buttons and navigation (`500`).
    Medium,
    /// Headings and key figures (`600`).
    Semibold,
}

impl FontWeight {
    /// All weights registered by [`install_fonts`].
    pub const ALL: [Self; 3] = [Self::Regular, Self::Medium, Self::Semibold];

    /// Numeric CSS-style weight (`400`, `500`, `600`).
    pub fn value(self) -> u16 {
        match self {
            Self::Regular => 400,
            Self::Medium => 500,
            Self::Semibold => 600,
        }
    }

    fn family_name(self) -> &'static str {
        match self {
            Self::Regular => "prism-400",
            Self::Medium => "prism-500",
            Self::Semibold => "prism-600",
        }
    }

    fn numeric_family_name(self) -> &'static str {
        match self {
            Self::Regular => "prism-num-400",
            Self::Medium => "prism-num-500",
            Self::Semibold => "prism-num-600",
        }
    }
}

/// Installs the bundled Inter typeface (three weights + tabular figures) into `ctx`.
///
/// Inter becomes the primary proportional font, so plain `egui` labels pick it up too. Existing
/// fonts are kept as fallbacks. Calling this more than once is cheap and has no effect.
///
/// Without the `inter` cargo feature this is a no-op and widgets keep using `egui`'s fonts.
pub fn install_fonts(ctx: &Context) {
    #[cfg(feature = "inter")]
    {
        if fonts_installed(ctx) {
            return;
        }
        for weight in FontWeight::ALL {
            let wght = f32::from(weight.value());
            let tweak = || FontTweak {
                coords: VariationCoords::new([(b"wght", wght)]),
                ..FontTweak::default()
            };

            let mut families = vec![
                InsertFontFamily {
                    family: FontFamily::Name(weight.family_name().into()),
                    priority: FontPriority::Highest,
                },
                InsertFontFamily {
                    family: FontFamily::Name(weight.numeric_family_name().into()),
                    priority: FontPriority::Lowest,
                },
            ];
            if weight == FontWeight::Regular {
                families.push(InsertFontFamily {
                    family: FontFamily::Proportional,
                    priority: FontPriority::Highest,
                });
            }
            ctx.add_font(FontInsert::new(
                &format!("prism-inter-{}", weight.value()),
                FontData::from_static(INTER_VARIABLE).tweak(tweak()),
                families,
            ));
            ctx.add_font(FontInsert::new(
                &format!("prism-inter-tnum-{}", weight.value()),
                FontData::from_static(INTER_TABULAR_DIGITS).tweak(tweak()),
                vec![InsertFontFamily {
                    family: FontFamily::Name(weight.numeric_family_name().into()),
                    priority: FontPriority::Highest,
                }],
            ));
        }
        ctx.data_mut(|d| d.insert_temp(Id::NULL.with(FONTS_INSTALLED_ID), true));
    }
    #[cfg(not(feature = "inter"))]
    {
        let _ = ctx;
    }
}

/// Returns `true` once [`install_fonts`] has registered the bundled Inter weights in `ctx`.
pub fn fonts_installed(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(Id::NULL.with(FONTS_INSTALLED_ID)))
        .unwrap_or(false)
}

/// Returns a [`FontId`] for Inter at `size` and `weight`, or the default proportional font
/// when the bundled fonts are not installed.
pub fn prism_font(ctx: &Context, size: f32, weight: FontWeight) -> FontId {
    if fonts_installed(ctx) {
        FontId::new(size, FontFamily::Name(weight.family_name().into()))
    } else {
        FontId::proportional(size)
    }
}

/// Like [`prism_font`], but with tabular (equal-width) digits so numbers line up in columns,
/// counters and live readouts. Non-digit characters use regular Inter.
pub fn prism_numeric_font(ctx: &Context, size: f32, weight: FontWeight) -> FontId {
    if fonts_installed(ctx) {
        FontId::new(size, FontFamily::Name(weight.numeric_family_name().into()))
    } else {
        FontId::proportional(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prism_font_falls_back_until_installed() {
        let ctx = Context::default();
        assert_eq!(
            prism_font(&ctx, 13.0, FontWeight::Medium),
            FontId::proportional(13.0)
        );
        install_fonts(&ctx);
        #[cfg(feature = "inter")]
        {
            assert!(fonts_installed(&ctx));
            assert_eq!(
                prism_font(&ctx, 13.0, FontWeight::Medium).family,
                FontFamily::Name("prism-500".into())
            );
            assert_eq!(
                prism_numeric_font(&ctx, 12.0, FontWeight::Regular).family,
                FontFamily::Name("prism-num-400".into())
            );
        }
    }

    #[test]
    fn installed_fonts_lay_out_text() {
        let ctx = Context::default();
        install_fonts(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            for weight in FontWeight::ALL {
                ui.label(egui::RichText::new("Render queue 1234").font(prism_font(
                    ui.ctx(),
                    13.0,
                    weight,
                )));
                ui.label(egui::RichText::new("74 / 240").font(prism_numeric_font(
                    ui.ctx(),
                    12.0,
                    weight,
                )));
            }
        });
        output.textures_delta.clear();
    }
}
