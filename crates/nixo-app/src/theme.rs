use std::path::PathBuf;

use egui::{Color32, Context, CornerRadius, FontId, Stroke};
use tracing::info;

use crate::app::{ACCENT_COLOR, BG_OLED_BLACK, NixoApp};

/// Fonte principal da UI (Inter), embutida no binário.
const INTER_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-Bold.ttf");
const LUCIDE: &[u8] = include_bytes!("../../../assets/fonts/lucide.ttf");

#[inline]
pub(crate) fn font_bold(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("bold".into()))
}

impl NixoApp {
    pub(crate) fn configure_fonts(ctx: &Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Inter é a fonte primária da UI (regular e bold).
        fonts.font_data.insert(
            "inter_regular".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(INTER_REGULAR)),
        );
        fonts.font_data.insert(
            "inter_bold".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(INTER_BOLD)),
        );
        fonts.font_data.insert(
            "lucide".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(LUCIDE)),
        );

        fonts.families.insert(
            egui::FontFamily::Name("lucide".into()),
            vec!["lucide".to_owned(), "inter_regular".to_owned()],
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "inter_regular".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("lucide".to_owned());

        // Fontes do sistema ficam apenas como fallback de glifos não cobertos pela Inter.
        let (regular_path, bold_path) = Self::find_system_fonts();

        let mut regular_loaded = false;
        if let Some(path) = regular_path
            && let Ok(bytes) = std::fs::read(&path)
        {
            info!("Carregando fonte nativa de fallback: {}", path.display());
            fonts.font_data.insert(
                "system_regular".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(1, "system_regular".to_owned());
            regular_loaded = true;
        }

        let mut bold_chain = vec!["inter_bold".to_owned()];
        if let Some(path) = bold_path
            && let Ok(bytes) = std::fs::read(&path)
        {
            info!(
                "Carregando fonte nativa bold de fallback: {}",
                path.display()
            );
            fonts.font_data.insert(
                "system_bold".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            bold_chain.push("system_bold".to_owned());
        }
        if regular_loaded {
            bold_chain.push("system_regular".to_owned());
        }
        bold_chain.push("Ubuntu-Light".to_owned());

        fonts
            .families
            .insert(egui::FontFamily::Name("bold".into()), bold_chain);

        Self::register_subtitle_preview_fonts(&mut fonts);

        ctx.set_fonts(fonts);
    }

    pub(crate) fn register_subtitle_preview_fonts(fonts: &mut egui::FontDefinitions) {
        let families: [(&str, &[u8], &[u8]); 6] = [
            ("Inter", INTER_REGULAR, INTER_BOLD),
            (
                "Oswald",
                include_bytes!("../../../assets/fonts/Oswald-Regular.ttf"),
                include_bytes!("../../../assets/fonts/Oswald-Bold.ttf"),
            ),
            (
                "Montserrat",
                include_bytes!("../../../assets/fonts/Montserrat-Regular.ttf"),
                include_bytes!("../../../assets/fonts/Montserrat-Bold.ttf"),
            ),
            (
                "Roboto",
                include_bytes!("../../../assets/fonts/Roboto-Regular.ttf"),
                include_bytes!("../../../assets/fonts/Roboto-Bold.ttf"),
            ),
            (
                "Open Sans",
                include_bytes!("../../../assets/fonts/OpenSans-Regular.ttf"),
                include_bytes!("../../../assets/fonts/OpenSans-Bold.ttf"),
            ),
            (
                "Lato",
                include_bytes!("../../../assets/fonts/Lato-Regular.ttf"),
                include_bytes!("../../../assets/fonts/Lato-Bold.ttf"),
            ),
        ];

        for (family_name, reg_bytes, bold_bytes) in families {
            let reg_key = format!("sub_reg_{family_name}");
            fonts.font_data.insert(
                reg_key.clone(),
                std::sync::Arc::new(egui::FontData::from_static(reg_bytes)),
            );
            let mut reg_chain = vec![reg_key.clone()];
            if fonts.font_data.contains_key("system_regular") {
                reg_chain.push("system_regular".to_owned());
            }
            reg_chain.push("Ubuntu-Light".to_owned());
            fonts
                .families
                .insert(egui::FontFamily::Name(reg_key.into()), reg_chain);

            let bold_key = format!("sub_bold_{family_name}");
            fonts.font_data.insert(
                bold_key.clone(),
                std::sync::Arc::new(egui::FontData::from_static(bold_bytes)),
            );
            let mut bold_chain = vec![bold_key.clone()];
            if fonts.font_data.contains_key("system_bold") {
                bold_chain.push("system_bold".to_owned());
            }
            bold_chain.push("Ubuntu-Light".to_owned());
            fonts
                .families
                .insert(egui::FontFamily::Name(bold_key.into()), bold_chain);
        }
    }

    pub(crate) fn subtitle_font_id(ctx: &Context, family: &str, bold: bool, size: f32) -> FontId {
        let key = if bold {
            format!("sub_bold_{family}")
        } else {
            format!("sub_reg_{family}")
        };
        let family_key = egui::FontFamily::Name(key.into());
        let has_family = ctx.fonts(|f| f.families().contains(&family_key));
        if has_family {
            FontId::new(size, family_key)
        } else if bold {
            font_bold(size)
        } else {
            FontId::proportional(size)
        }
    }

    fn find_system_fonts() -> (Option<PathBuf>, Option<PathBuf>) {
        #[cfg(target_os = "windows")]
        {
            let windir = std::env::var("WINDIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(r"C:\Windows"));
            let fonts_dir = windir.join("Fonts");

            let candidates = [
                ("segoeui.ttf", "segoeuib.ttf"),
                ("arial.ttf", "arialbd.ttf"),
            ];

            for (reg, bld) in candidates {
                let reg_p = fonts_dir.join(reg);
                let bld_p = fonts_dir.join(bld);
                if reg_p.exists() && bld_p.exists() {
                    return (Some(reg_p), Some(bld_p));
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let candidates = [
                (
                    "/System/Library/Fonts/SFNS.ttf",
                    "/System/Library/Fonts/SFNSBold.ttf",
                ),
                (
                    "/System/Library/Fonts/SFCompact.ttf",
                    "/System/Library/Fonts/SFCompactBold.ttf",
                ),
                ("/Library/Fonts/Arial.ttf", "/Library/Fonts/Arial Bold.ttf"),
            ];

            for (reg, bld) in candidates {
                let reg_p = PathBuf::from(reg);
                let bld_p = PathBuf::from(bld);
                if reg_p.exists() && bld_p.exists() {
                    return (Some(reg_p), Some(bld_p));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidates = [
                (
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                ),
                (
                    "/usr/share/fonts/TTF/DejaVuSans.ttf",
                    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
                ),
                (
                    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
                    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
                ),
                (
                    "/usr/share/fonts/noto/NotoSans-Regular.ttf",
                    "/usr/share/fonts/noto/NotoSans-Bold.ttf",
                ),
            ];

            for (reg, bld) in candidates {
                let reg_p = PathBuf::from(reg);
                let bld_p = PathBuf::from(bld);
                if reg_p.exists() && bld_p.exists() {
                    return (Some(reg_p), Some(bld_p));
                }
            }
        }

        (None, None)
    }

    pub(crate) fn configure_theme(ctx: &Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = Color32::from_rgba_unmultiplied(20, 22, 28, 248);
        visuals.window_stroke =
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
        visuals.window_corner_radius = CornerRadius::same(99);
        visuals.popup_shadow = egui::Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: Color32::from_black_alpha(130),
        };
        visuals.menu_corner_radius = CornerRadius::same(12);
        visuals.extreme_bg_color = BG_OLED_BLACK;
        visuals.selection.bg_fill = ACCENT_COLOR;
        visuals.widgets.active.bg_fill = ACCENT_COLOR;
        visuals.widgets.active.bg_stroke = Stroke::NONE;
        visuals.widgets.active.corner_radius = CornerRadius::same(99);
        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.corner_radius = CornerRadius::same(99);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 41, 52);
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.corner_radius = CornerRadius::same(99);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 28, 35);
        visuals.widgets.noninteractive.bg_stroke =
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 25));
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(99);
        ctx.set_visuals(visuals);

        ctx.style_mut(|style| {
            style.spacing.slider_rail_height = 3.0;
            style.interaction.tooltip_delay = 0.12;
            style.spacing.scroll.bar_outer_margin = 0.0;
            style.spacing.scroll.handle_min_length = 24.0;
        });
    }

    /// Customiza a barra de título nativa no Windows para o cinza escuro elegante #222222 com texto branco.
    #[cfg(target_os = "windows")]
    pub(crate) fn apply_windows_titlebar_color() -> bool {
        type Hwnd = *mut std::ffi::c_void;
        type Bool = i32;
        type FnFindWindow = unsafe extern "system" fn(*const u16, *const u16) -> Hwnd;
        type FnGetWindowThreadProcessId = unsafe extern "system" fn(Hwnd, *mut u32) -> u32;
        type FnSetWindowPos =
            unsafe extern "system" fn(Hwnd, Hwnd, i32, i32, i32, i32, u32) -> Bool;
        type FnDwmSetWindowAttribute =
            unsafe extern "system" fn(Hwnd, u32, *const std::ffi::c_void, u32) -> i32;

        // SAFETY: Carrega a biblioteca user32.dll padrão do Windows.
        let user32 = match unsafe { libloading::Library::new("user32.dll") } {
            Ok(lib) => lib,
            Err(_) => return false,
        };

        // SAFETY: Carrega a biblioteca dwmapi.dll padrão do Windows.
        let dwmapi = match unsafe { libloading::Library::new("dwmapi.dll") } {
            Ok(lib) => lib,
            Err(_) => return false,
        };

        // SAFETY: Assinaturas de funções exportadas oficiais da Win32 API.
        unsafe {
            let find_window: Result<libloading::Symbol<FnFindWindow>, _> =
                user32.get(b"FindWindowW");
            let get_window_thread_process_id: Result<
                libloading::Symbol<FnGetWindowThreadProcessId>,
                _,
            > = user32.get(b"GetWindowThreadProcessId");
            let set_window_pos: Result<libloading::Symbol<FnSetWindowPos>, _> =
                user32.get(b"SetWindowPos");
            let set_attrib: Result<libloading::Symbol<FnDwmSetWindowAttribute>, _> =
                dwmapi.get(b"DwmSetWindowAttribute");

            if let (Ok(find_window), Ok(get_pid), Ok(set_window_pos), Ok(set_attrib)) = (
                find_window,
                get_window_thread_process_id,
                set_window_pos,
                set_attrib,
            ) {
                let title: Vec<u16> = "Nixo Media Player\0".encode_utf16().collect();
                let hwnd = find_window(std::ptr::null(), title.as_ptr());
                if hwnd.is_null() {
                    return false;
                }

                let mut proc_id = 0u32;
                get_pid(hwnd, &raw mut proc_id);
                if proc_id != std::process::id() {
                    return false;
                }

                // 1. DWMWA_USE_IMMERSIVE_DARK_MODE (20 e 19)
                let dark: u32 = 1;
                let _ = set_attrib(hwnd, 20, (&raw const dark).cast::<std::ffi::c_void>(), 4);
                let _ = set_attrib(hwnd, 19, (&raw const dark).cast::<std::ffi::c_void>(), 4);

                // 2. DWMWA_CAPTION_COLOR (35) = 0x00222222 (Cinza Escuro #222222)
                let dark_gray_caption: u32 = 0x00222222;
                let _ = set_attrib(
                    hwnd,
                    35,
                    (&raw const dark_gray_caption).cast::<std::ffi::c_void>(),
                    4,
                );

                // 3. DWMWA_TEXT_COLOR (36) = 0x00FFFFFF (Branco Puro #FFFFFF)
                let white_text: u32 = 0x00FFFFFF;
                let _ = set_attrib(
                    hwnd,
                    36,
                    (&raw const white_text).cast::<std::ffi::c_void>(),
                    4,
                );

                // 4. DWMWA_BORDER_COLOR (34) = 0x00222222 (#222222 para harmonia com a barra)
                let dark_gray_border: u32 = 0x00222222;
                let _ = set_attrib(
                    hwnd,
                    34,
                    (&raw const dark_gray_border).cast::<std::ffi::c_void>(),
                    4,
                );

                // 5. DWMWA_SYSTEMBACKDROP_TYPE (38) = 3 (Acrylic blur nativo no Windows 11)
                let acrylic_backdrop: u32 = 3;
                let _ = set_attrib(
                    hwnd,
                    38,
                    (&raw const acrylic_backdrop).cast::<std::ffi::c_void>(),
                    4,
                );

                // 6. Força o DWM e o Windows a redesenharem a moldura/barra de título imediatamente (SWP_FRAMECHANGED)
                const SWP_FRAMECHANGED: u32 = 0x0020;
                const SWP_NOMOVE: u32 = 0x0002;
                const SWP_NOSIZE: u32 = 0x0001;
                const SWP_NOZORDER: u32 = 0x0004;
                const SWP_NOACTIVATE: u32 = 0x0010;
                let _ = set_window_pos(
                    hwnd,
                    std::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );

                true
            } else {
                false
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn apply_windows_titlebar_color() -> bool {
        true
    }
}
